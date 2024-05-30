variable "role_arn" {
  description = "The ARN of the role to assume in the secondary account to provision resources ('account_b' IAM role)"
  type        = string
  default     = "arn:aws:iam::361548816046:role/terraform"
}
