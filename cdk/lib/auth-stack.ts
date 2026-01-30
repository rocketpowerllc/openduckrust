import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';

export interface AuthStackProps extends cdk.StackProps {
  // Add custom props here
  [key: string]: any;
}

export class AuthStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: AuthStackProps) {
    super(scope, id, props);

    // TODO: Implement auth resources
    // See product.architecture.md for design details
  }
}
