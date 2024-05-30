# Infra

## Prerequisites

Ensure that you have installed the following tools locally:

- [awscli](https://docs.aws.amazon.com/cli/latest/userguide/install-cliv2.html)
- [kubectl](https://Kubernetes.io/docs/tasks/tools/)
- [terraform](https://learn.hashicorp.com/tutorials/terraform/install-cli)

## Deploy

1. To provision the example provided, execute the following commands:

    ```sh
    terraform init -upgrade
    terraform apply
    ```

2. The CLI will prompt you for an IAM role that your current IAM identity can assume in a separate account to create the IAM role that is to be assumed by the EKS Pod Identity IAM role created. Enter this IAM role ARN and hit `enter`

3. Terraform will generate a plan output and prompt you for a `yes` or `no` to execute - enter `yes` and hit `enter`.

4. Once all of the resources have successfully been provisioned, the following command can be used to update the `kubeconfig`  on your local machine and allow you to interact with the EKS Cluster using `kubectl`:

    ```sh
    aws eks --region us-east-1 update-kubeconfig --name epicac
    ```
4. During the Terraform apply, the following files were updated with the necessary details for deploying the example into your cluster to witness/validate `epicac`:
    - The `aws-config` file in the root of the project was updated with the IAM role ARN that will be assumed in the destination account
    - The `build.sh` script in the root of the project was updated with the URL of the ECR repository created

5. From the project root directory, execute the following to build, tag, and push the sample container image defined in the `Dockerfile` to the ECR repository created. Executing this script will also update the `pod.yaml` with the image that will be uploaded to ECR:

    ```sh
    ./build.sh
    ```
6. From the project root directory, execute the following to deploy the example pod that uses `epicac` to assume a role in the destination account and perform an `aws s3 ls` operation using the AWS Boto3 Python SDK:

    ```sh
    kubectl apply -f pod.yaml
    ```

You can view the logged output with:

    ```sh
    kubectl logs -n epicac epicac
    ```

## Destroy

To teardown and remove the resources created in this example, execute the following commands:

```sh
terraform destroy -auto-approve
```
