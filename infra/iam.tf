################################################################################
# Pod Identity Role - Account A
# Created in the same account as the cluster
################################################################################

module "pod_identity_account_a" {
  source  = "terraform-aws-modules/eks-pod-identity/aws"
  version = "~> 1.0"

  name = "${local.name}-account-a"

  associations = {
    account-b = {
      cluster_name    = module.eks.cluster_name
      namespace       = "epicac"
      service_account = "epicac"
    }
  }

  attach_custom_policy = true
  policy_statements = [
    {
      sid = "AssumeRoleAccountB"
      actions = [
        "sts:AssumeRole",
        "sts:TagSession",
      ]
      resources = [module.iam_role_account_b.iam_role_arn]
    }
  ]

  tags = module.tags.tags
}

################################################################################
# IAM Role - Account B
################################################################################

module "iam_role_account_b" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-assumable-role"
  version = "~> 5.0"

  providers = {
    aws = aws.account_b
  }

  role_name         = "${local.name}-account-b"
  create_role       = true
  role_requires_mfa = false

  trusted_role_arns       = [module.pod_identity_account_a.iam_role_arn]
  custom_role_policy_arns = ["arn:aws:iam::aws:policy/AmazonS3ReadOnlyAccess"]

  tags = module.tags.tags
}

resource "local_file" "aws_config" {
  content = <<-EOT
    [profile dest_role]
    source_profile = src_role
    role_arn=${module.iam_role_account_b.iam_role_arn}

    [profile src_role]
    credential_process = /opt/epicac
  EOT

  filename = "${path.module}/../aws-config"
}
