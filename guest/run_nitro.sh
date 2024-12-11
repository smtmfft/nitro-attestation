# build docker
docker build -t nitro-test -f docker/Dockerfile .

# make elf
nitro-cli build-enclave --docker-uri nitro-test:latest --output-file test.elf
# run elf
nitro-cli run-enclave --eif-path test.elf --cpu-count 2 --memory 4096 --attach-console

# describe enclave
nitro-cli describe-enclaves

# check output
nitro-cli console --enclave-id $(nitro-cli describe-enclaves | jq -r '.[0].EnclaveID')