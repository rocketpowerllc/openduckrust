import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';

export interface NetworkStackProps extends cdk.StackProps {
  // Add custom props here
  [key: string]: any;
}

export class NetworkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: NetworkStackProps) {
    super(scope, id, props);

    // TODO: Implement network resources
    // See product.architecture.md for design details
  }
}
