import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';

export interface FrontendStackProps extends cdk.StackProps {
  // Add custom props here
  [key: string]: any;
}

export class FrontendStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: FrontendStackProps) {
    super(scope, id, props);

    // TODO: Implement frontend resources
    // See product.architecture.md for design details
  }
}
