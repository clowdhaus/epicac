terraform {
  required_version = "~> 1.3"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.37"
    }
  }

  backend "s3" {
    bucket         = "clowd-haus-iac-us-east-1"
    key            = "epicac/us-east-1/terraform.tfstate"
    region         = "us-east-1"
    dynamodb_table = "clowd-haus-terraform-state"
    encrypt        = true
  }
}

provider "aws" {
  region = local.region
}

provider "aws" {
  alias = "account-b"
  assume_role {
    role_arn     = "<TODO>"
    session_name = local.name
  }

  region = local.region
}

################################################################################
# Common Locals
################################################################################

locals {
  name        = "epicac"
  region      = "us-east-1"
  environment = "nonprod"

  vpc_cidr = "10.0.0.0/16"
  azs      = slice(data.aws_availability_zones.available.names, 0, 3)
}

################################################################################
# Common Data
################################################################################

data "aws_caller_identity" "current" {}
data "aws_availability_zones" "available" {}

################################################################################
# Common Modules
################################################################################

module "tags" {
  source  = "clowdhaus/tags/aws"
  version = "~> 1.0"

  application = local.name
  environment = local.environment
  repository  = "https://github.com/clowdhaus/epicac"
}
