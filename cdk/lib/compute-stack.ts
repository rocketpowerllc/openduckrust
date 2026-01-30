import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';

export interface ComputeStackProps extends cdk.StackProps {
  // Add custom props here
  [key: string]: any;
}

export class ComputeStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ComputeStackProps) {
    super(scope, id, props);

    // TODO: Implement compute resources
    // See product.architecture.md for design details
  }
}
