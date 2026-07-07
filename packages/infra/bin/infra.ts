#!/usr/bin/env node
import * as cdk from "aws-cdk-lib";

import { PROJECT_NAME } from "../constants";
import { DnsStack } from "../lib/DnsStack";
import { LandingStack } from "../lib/LandingStack";
import { ReleasesStack } from "../lib/ReleasesStack";

const app = new cdk.App();

const baseProps = {
  env: {
    account: process.env.CDK_DEFAULT_ACCOUNT,
    region: process.env.CDK_DEFAULT_REGION,
  },
};

const dnsStack = new DnsStack(app, `${PROJECT_NAME}DnsStack`, baseProps);

const releasesStack = new ReleasesStack(app, `${PROJECT_NAME}ReleasesStack`, {
  ...baseProps,
  hostedZone: dnsStack.hostedZone,
  certificate: dnsStack.certificate,
});
releasesStack.addDependency(dnsStack);

const landingStack = new LandingStack(app, `${PROJECT_NAME}LandingStack`, {
  ...baseProps,
  hostedZone: dnsStack.hostedZone,
  certificate: dnsStack.certificate,
});
landingStack.addDependency(dnsStack);

app.synth();
