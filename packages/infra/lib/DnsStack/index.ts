import * as cdk from "aws-cdk-lib";
import {
  Certificate,
  CertificateValidation,
  ICertificate,
} from "aws-cdk-lib/aws-certificatemanager";
import { HostedZone, IHostedZone } from "aws-cdk-lib/aws-route53";
import { Construct } from "constructs";

import { APP_DOMAIN, ROOT_DOMAIN, WWW_APP_DOMAIN } from "../../constants";

export class DnsStack extends cdk.Stack {
  public readonly hostedZone: IHostedZone;
  public readonly certificate: ICertificate;

  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const hostedZoneId =
      this.node.tryGetContext("hostedZoneId") ?? process.env.CLARUS_HOSTED_ZONE_ID;

    if (!hostedZoneId) {
      throw new Error(
        "Missing hosted zone id. Pass --context hostedZoneId=... or set CLARUS_HOSTED_ZONE_ID.",
      );
    }

    this.hostedZone = HostedZone.fromHostedZoneAttributes(this, "HostedZone", {
      hostedZoneId,
      zoneName: ROOT_DOMAIN,
    });

    this.certificate = new Certificate(this, "DomainCertificate", {
      domainName: APP_DOMAIN,
      subjectAlternativeNames: [WWW_APP_DOMAIN, `*.${APP_DOMAIN}`],
      validation: CertificateValidation.fromDns(this.hostedZone),
    });

    new cdk.CfnOutput(this, "CertificateArn", {
      value: this.certificate.certificateArn,
    });
  }
}
