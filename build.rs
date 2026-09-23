use ci_utils::ci_generator::CiGenerator;

fn main() {
    CiGenerator::new(env!("CARGO_PKG_NAME"))
        .as_basic_service()
        // Spelled out: the default is `ghcr.io/${{ github.repository }}`, and the owner
        // `MyJetTools` is mixed case, which `docker build -t` refuses.
        .set_docker_image_name("ghcr.io/myjettools/my-no-sql-node")
        .add_docker_copy_file("./wwwroot", "./wwwroot")
        .generate_github_ci_file()
        .with_ci_test()
        .build();
}
