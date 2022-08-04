# Client for accessing secret in Vault
Set the secret store with:
```bash
$  cat example/test.json 
{
  "token" : "hvs.CAESIFdXlCrwRH4-lqw9PAxJJUeDguKWPD7Cmsl0abwCb0IHGh4KHGh2cy5EdU9Vd1J2akpXZkxjbU1JN0Zxc0FZaDY",
  "url": "http://127.0.0.1:8200",
  "path": "guestowner1/workload-id/secret"
}
$ cd example
$ curl -H "Content-Type: application/json" \
	-d  @test.json \
	-X POST \
	http://127.0.0.1:8000/secret-store/update
```
The function `pub async fn get_secret_from_vault()` return the Secret stored in the vault database.


## Testing
```bash
$ secret-db-vault/start-vault.sh
$ secret-db-vault/create-secret.sh
[...]
hvs.CAESIOGRWPRVBVupBrvFiAgRb09lLiap18sh3ASTRnwljtg6Gh4KHGh2cy5QMktTbXkxT1R2Smkxd29YbGtSRGk2VkQ
$ export VAULT_TOKEN="hvs.CAESIOGRWPRVBVupBrvFiAgRb09lLiap18sh3ASTRnwljtg6Gh4KHGh2cy5QMktTbXkxT1R2Smkxd29YbGtSRGk2VkQ"
$ export VAULT_ADDR="http://127.0.0.1:8200"
$ cargo test
```
