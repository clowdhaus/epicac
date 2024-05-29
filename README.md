# epicac

Amazon EKS Pod Identity cross-account credential process (role chaining)

```sh
curl -H "Authorization: $(cat $AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE)" $AWS_CONTAINER_CREDENTIALS_FULL_URI | \
  jq -c '{AccessKeyId: .AccessKeyId, SecretAccessKey: .SecretAccessKey, SessionToken: .Token, Expiration: .Expiration, Version: 1}'
```

```sh
curl -H "Authorization: $(cat $AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE)" $AWS_CONTAINER_CREDENTIALS_FULL_URI | jq
```

Returns

```json
{
  "AccessKeyId": "xxx",
  "SecretAccessKey": "xxx",
  "Token": "xxx",
  "AccountId": "xxx",
  "Expiration": "2024-05-29T20:37:18Z"
}
```

<https://stackoverflow.com/questions/24689601/stdiotcpstreamread-as-string-returns-an-empty-string?rq=3>
