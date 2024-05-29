################################################################################
# Pod Identity Role - Account A
# Created in the same account as the cluster
################################################################################

module "pod_identity_account_a" {
  source  = "terraform-aws-modules/eks-pod-identity/aws"
  version = "~> 1.0"

  name = "${loca.name}-account-a"

  attach_custom_policy = true
  policy_statements = [
    {
      sid       = "AssumeRoleAccountB"
      actions   = ["sts:AssumeRole"]
      resources = [module.iam_role_account_b.iam_role_arn]
    }
  ]

  tags = module.tags
}

################################################################################
# IAM Role - Account B
################################################################################

module "iam_role_account_b" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-assumable-role"
  version = "~> 5.0"

  role_name = "${local.name}-account-b"

  trusted_role_arns       = [module.pod_identity_account_a.role_arn]
  custom_role_policy_arns = ["arn:aws:iam::aws:policy/AmazonS3ReadOnlyAccess"]

  tags = module.tags.tags
}
