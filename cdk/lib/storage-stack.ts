import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';

export interface StorageStackProps extends cdk.StackProps {
  // Add custom props here
  [key: string]: any;
}

export class StorageStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: StorageStackProps) {
    super(scope, id, props);

    // TODO: Implement storage resources
    // See product.architecture.md for design details
  }
}
