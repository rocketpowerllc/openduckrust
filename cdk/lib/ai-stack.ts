import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';

export interface AiStackProps extends cdk.StackProps {
  // Add custom props here
  [key: string]: any;
}

export class AiStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: AiStackProps) {
    super(scope, id, props);

    // TODO: Implement ai resources
    // See product.architecture.md for design details
  }
}
