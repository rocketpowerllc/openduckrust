#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { NetworkStack } from '../lib/network-stack';
import { AuthStack } from '../lib/auth-stack';
import { ComputeStack } from '../lib/compute-stack';
import { StorageStack } from '../lib/storage-stack';
import { FrontendStack } from '../lib/frontend-stack';
import { AiStack } from '../lib/ai-stack';

const app = new cdk.App();
const env = { account: process.env.CDK_DEFAULT_ACCOUNT, region: 'us-east-1' };
const productName = 'openduckrust';

const network = new NetworkStack(app, `${productName}-network`, { env });
const auth    = new AuthStack(app, `${productName}-auth`, { env });
const storage = new StorageStack(app, `${productName}-storage`, { env, enableDax: false, enableOpenSearch: false });
const compute = new ComputeStack(app, `${productName}-compute`, { env, vpc: network.vpc });
const frontend = new FrontendStack(app, `${productName}-frontend`, { env, domain: 'openduckrust.com' });
const ai      = new AiStack(app, `${productName}-ai`, { env });
