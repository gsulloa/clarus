import * as cdk from "aws-cdk-lib";
import * as cloudfront from "aws-cdk-lib/aws-cloudfront";
import * as origins from "aws-cdk-lib/aws-cloudfront-origins";
import * as iam from "aws-cdk-lib/aws-iam";
import { ICertificate } from "aws-cdk-lib/aws-certificatemanager";
import { ARecord, AaaaRecord, IHostedZone, RecordTarget } from "aws-cdk-lib/aws-route53";
import { CloudFrontTarget } from "aws-cdk-lib/aws-route53-targets";
import * as s3 from "aws-cdk-lib/aws-s3";
import * as ssm from "aws-cdk-lib/aws-ssm";
import { Construct } from "constructs";

import {
  APP_PUBLIC_URL,
  GITHUB_REPOSITORY,
  PROJECT_NAME,
  RELEASES_DOMAIN,
  RELEASES_RECORD_NAME,
} from "../../constants";

export interface ReleasesStackProps extends cdk.StackProps {
  hostedZone: IHostedZone;
  certificate: ICertificate;
}

export class ReleasesStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ReleasesStackProps) {
    super(scope, id, props);

    const bucket = new s3.Bucket(this, "ArtifactsBucket", {
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      enforceSSL: true,
      encryption: s3.BucketEncryption.S3_MANAGED,
      versioned: true,
      removalPolicy: cdk.RemovalPolicy.RETAIN,
      autoDeleteObjects: false,
    });

    const origin = origins.S3BucketOrigin.withOriginAccessControl(bucket);

    const manifestHeaders = new cloudfront.ResponseHeadersPolicy(this, "ManifestHeaders", {
      comment: "CORS and no-cache headers for Clarus release manifests",
      corsBehavior: {
        accessControlAllowCredentials: false,
        accessControlAllowHeaders: ["*"],
        accessControlAllowMethods: ["GET", "HEAD"],
        accessControlAllowOrigins: [APP_PUBLIC_URL],
        originOverride: true,
      },
      customHeadersBehavior: {
        customHeaders: [
          {
            header: "Cache-Control",
            value: "no-cache, max-age=0",
            override: true,
          },
        ],
      },
    });

    const distribution = new cloudfront.Distribution(this, "Distribution", {
      defaultBehavior: {
        origin,
        viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
        compress: true,
        cachePolicy: cloudfront.CachePolicy.CACHING_OPTIMIZED,
      },
      additionalBehaviors: {
        "latest.json": {
          origin,
          viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
          cachePolicy: cloudfront.CachePolicy.CACHING_DISABLED,
          responseHeadersPolicy: manifestHeaders,
        },
        "download.json": {
          origin,
          viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
          cachePolicy: cloudfront.CachePolicy.CACHING_DISABLED,
          responseHeadersPolicy: manifestHeaders,
        },
      },
      domainNames: [RELEASES_DOMAIN],
      certificate: props.certificate,
      httpVersion: cloudfront.HttpVersion.HTTP2_AND_3,
    });

    const target = RecordTarget.fromAlias(new CloudFrontTarget(distribution));
    new ARecord(this, "ReleasesAliasA", {
      zone: props.hostedZone,
      recordName: RELEASES_RECORD_NAME,
      target,
    });
    new AaaaRecord(this, "ReleasesAliasAaaa", {
      zone: props.hostedZone,
      recordName: RELEASES_RECORD_NAME,
      target,
    });

    const existingProviderArn = this.node.tryGetContext("githubOidcProviderArn") as
      | string
      | undefined;

    const oidcProvider = existingProviderArn
      ? iam.OpenIdConnectProvider.fromOpenIdConnectProviderArn(
          this,
          "GithubOidcProvider",
          existingProviderArn,
        )
      : new iam.OpenIdConnectProvider(this, "GithubOidcProvider", {
          url: "https://token.actions.githubusercontent.com",
          clientIds: ["sts.amazonaws.com"],
        });

    const publishRole = new iam.Role(this, "PublishRole", {
      assumedBy: new iam.WebIdentityPrincipal(oidcProvider.openIdConnectProviderArn, {
        StringEquals: {
          "token.actions.githubusercontent.com:aud": "sts.amazonaws.com",
        },
        StringLike: {
          "token.actions.githubusercontent.com:sub": `repo:${GITHUB_REPOSITORY}:ref:refs/tags/v*`,
        },
      }),
      description: "Assumed by GitHub Actions to publish Clarus release artifacts",
    });

    publishRole.addToPolicy(
      new iam.PolicyStatement({
        sid: "S3Objects",
        actions: ["s3:PutObject", "s3:GetObject"],
        resources: [`${bucket.bucketArn}/*`],
      }),
    );
    publishRole.addToPolicy(
      new iam.PolicyStatement({
        sid: "S3List",
        actions: ["s3:ListBucket"],
        resources: [bucket.bucketArn],
      }),
    );
    publishRole.addToPolicy(
      new iam.PolicyStatement({
        sid: "CloudFrontInvalidation",
        actions: ["cloudfront:CreateInvalidation"],
        resources: [
          `arn:aws:cloudfront::${this.account}:distribution/${distribution.distributionId}`,
        ],
      }),
    );

    // SSM parameters let the local .envrc resolve release infra dynamically
    // (same pattern as Argus/TokenWatch), so bucket/distribution/role are never
    // hardcoded in the local environment.
    new ssm.StringParameter(this, "ReleaseBucketNameParam", {
      parameterName: `/${PROJECT_NAME}/releases/bucket-name`,
      stringValue: bucket.bucketName,
    });
    new ssm.StringParameter(this, "ReleaseDistributionIdParam", {
      parameterName: `/${PROJECT_NAME}/releases/distribution-id`,
      stringValue: distribution.distributionId,
    });
    new ssm.StringParameter(this, "PublishRoleArnParam", {
      parameterName: `/${PROJECT_NAME}/releases/publish-role-arn`,
      stringValue: publishRole.roleArn,
    });

    new cdk.CfnOutput(this, "ReleaseBucketName", { value: bucket.bucketName });
    new cdk.CfnOutput(this, "ReleaseDistributionId", {
      value: distribution.distributionId,
    });
    new cdk.CfnOutput(this, "PublishRoleArn", { value: publishRole.roleArn });
    new cdk.CfnOutput(this, "ReleasePublicUrl", {
      value: `https://${RELEASES_DOMAIN}`,
    });
    cdk.Tags.of(this).add("Project", PROJECT_NAME);
  }
}
