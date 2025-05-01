FROM ubuntu:22.04
COPY ./target/release/my-no-sql-node ./target/release/my-no-sql-node 
COPY ./wwwroot ./wwwroot 
ENTRYPOINT ["./target/release/my-no-sql-node"]