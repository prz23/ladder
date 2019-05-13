function gen_protos () {
    find ./proto -name "*.proto" | while read protofile; do
        protoc ${protofile} --proto_path ./proto --rust_out ./src
    done
}

function run () {
    gen_protos
}

run