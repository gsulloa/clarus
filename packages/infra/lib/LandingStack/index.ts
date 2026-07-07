import * as path from "path";

import * as cdk from "aws-cdk-lib";
import { ICertificate } from "aws-cdk-lib/aws-certificatemanager";
import * as cloudfront from "aws-cdk-lib/aws-cloudfront";
import * as origins from "aws-cdk-lib/aws-cloudfront-origins";
import { ARecord, AaaaRecord, IHostedZone, RecordTarget } from "aws-cdk-lib/aws-route53";
import { CloudFrontTarget } from "aws-cdk-lib/aws-route53-targets";
import * as s3 from "aws-cdk-lib/aws-s3";
import { BucketDeployment, Source } from "aws-cdk-lib/aws-s3-deployment";
import { Construct } from "constructs";

import {
  APP_DOMAIN,
  APP_RECORD_NAME,
  PROJECT_NAME,
  WWW_APP_DOMAIN,
} from "../../constants";

export interface LandingStackProps extends cdk.StackProps {
  hostedZone: IHostedZone;
  certificate: ICertificate;
}

export class LandingStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: LandingStackProps) {
    super(scope, id, props);

    const bucket = new s3.Bucket(this, "SiteBucket", {
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      enforceSSL: true,
      encryption: s3.BucketEncryption.S3_MANAGED,
      removalPolicy: cdk.RemovalPolicy.RETAIN,
      autoDeleteObjects: false,
    });

    const distribution = new cloudfront.Distribution(this, "Distribution", {
      defaultRootObject: "index.html",
      defaultBehavior: {
        origin: origins.S3BucketOrigin.withOriginAccessControl(bucket),
        viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
        compress: true,
        cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
      },
      domainNames: [APP_DOMAIN, WWW_APP_DOMAIN],
      certificate: props.certificate,
      httpVersion: cloudfront.HttpVersion.HTTP2_AND_3,
      errorResponses: [
        {
          httpStatus: 403,
          responseHttpStatus: 200,
          responsePagePath: "/index.html",
          ttl: cdk.Duration.minutes(5),
        },
        {
          httpStatus: 404,
          responseHttpStatus: 200,
          responsePagePath: "/index.html",
          ttl: cdk.Duration.minutes(5),
        },
      ],
    });

    const target = RecordTarget.fromAlias(new CloudFrontTarget(distribution));
    new ARecord(this, "LandingAliasA", {
      zone: props.hostedZone,
      recordName: APP_RECORD_NAME,
      target,
    });
    new AaaaRecord(this, "LandingAliasAaaa", {
      zone: props.hostedZone,
      recordName: APP_RECORD_NAME,
      target,
    });
    new ARecord(this, "LandingWwwAliasA", {
      zone: props.hostedZone,
      recordName: `www.${APP_RECORD_NAME}`,
      target,
    });
    new AaaaRecord(this, "LandingWwwAliasAaaa", {
      zone: props.hostedZone,
      recordName: `www.${APP_RECORD_NAME}`,
      target,
    });

    new BucketDeployment(this, "DeployLanding", {
      sources: [Source.asset(path.resolve(__dirname, "../../static/landing"))],
      destinationBucket: bucket,
      distribution,
      distributionPaths: ["/*"],
    });

    new cdk.CfnOutput(this, "LandingBucketName", { value: bucket.bucketName });
    new cdk.CfnOutput(this, "LandingDistributionId", {
      value: distribution.distributionId,
    });
    new cdk.CfnOutput(this, "LandingPublicUrl", { value: `https://${APP_DOMAIN}` });
    cdk.Tags.of(this).add("Project", PROJECT_NAME);
  }
}
