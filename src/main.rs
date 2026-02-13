use mqtt_adapt::server::Server;
use tracing::Level;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    // 初始化tracing日志系统
    let _num_cpus = num_cpus::get();
    init_tracing();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        // 服务器地址

        // 创建服务器
        let server = Server::new("127.0.0.1:1883".parse().unwrap());

        // 启动服务器
        server.start().await;
    });
}
fn init_tracing() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
}
