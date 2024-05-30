################################################################################
# ECR Repository
################################################################################

module "ecr" {
  source  = "terraform-aws-modules/ecr/aws"
  version = "~> 1.6"

  repository_name = local.name

  repository_force_delete         = true # For example only
  create_lifecycle_policy         = false
  repository_image_tag_mutability = "MUTABLE"

  tags = module.tags.tags
}

################################################################################
# Image Build & Push Script
################################################################################

resource "local_file" "build_script" {
  content = <<-EOT
    #!/usr/bin/env bash

    aws ecr get-login-password --region ${local.region} | docker login --username AWS --password-stdin ${data.aws_caller_identity.current.account_id}.dkr.ecr.${local.region}.amazonaws.com

    TAG=epicac-$(date +%Y%m%d_%H%M%S)

    docker build --platform amd64 -t ${module.ecr.repository_url}:$${TAG} .
    docker push ${module.ecr.repository_url}:$${TAG}

    # Update the pod manifest with the new image tag
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
      sed -i "s|image:.*|image: ${module.ecr.repository_url}:$${TAG}|g" infra/pod.yaml
    elif [[ "$OSTYPE" == "darwin"* ]]; then
      sed -i '' "s|image:.*|image: ${module.ecr.repository_url}:$${TAG}|g" infra/pod.yaml
    else
      echo "Unsupported OS: $OSTYPE"
      exit 1
    fi
  EOT

  filename = "${path.module}/../build.sh"
}
