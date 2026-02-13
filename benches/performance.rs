use criterion::{black_box, Criterion};
use mqtt_adapt::topic::TopicManager;
use mqtt_adapt::routing::{router::MessageRouter, event::Event};
use mqtt_adapt::protocol::{MqttPacket, SubscribePacket, PublishPacket, Packet};
use bytes::{BytesMut, Bytes, BufMut};
use flume::{unbounded};
use tokio::runtime::Runtime;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use rumqttc::{AsyncClient, Client, MqttOptions, QoS};
use rand::Rng;
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkResult {
    timestamp: u64,
    benchmark_name: String,
    mean_time_ns: f64,
    std_dev_ns: f64,
    median_ns: f64,
    min_ns: f64,
    max_ns: f64,
    iterations: u64,
}

fn save_result(result: BenchmarkResult) {
    let benches_dir = Path::new("benches/results");
    if !benches_dir.exists() {
        std::fs::create_dir_all(benches_dir).expect("Failed to create results directory");
    }
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let filename = benches_dir.join(format!("benchmark_{}.json", timestamp));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&filename)
        .expect("Failed to open results file");
    
    writeln!(file, "{}", serde_json::to_string_pretty(&result).unwrap())
        .expect("Failed to write result");
    
    println!("Benchmark result saved to: {}", filename.display());
}

fn create_criterion() -> Criterion {
    Criterion::default()
        .save_baseline("baseline.json".into())
        .output_directory(Path::new("benches/results"))
}

fn main() {
    let mut criterion = create_criterion();
    bench_rumqttc_client(&mut criterion);

    criterion.final_summary();
 
}



// 测试rumqttc客户端性能
fn bench_rumqttc_client(c: &mut Criterion) {
    c.bench_function("rumqttc_connect_publish_disconnect", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            rt.spawn(async {
                let client_id = format!("rumqttc_client_{}", black_box(rand::random::<u64>()));
                let mut mqtt_options = MqttOptions::new(client_id, "127.0.0.1", 1883);
                mqtt_options.set_transport(rumqttc::Transport::Tcp);
                mqtt_options.set_keep_alive(std::time::Duration::from_secs(5));
                
                let (mut client, mut connection) = AsyncClient::new(mqtt_options, 10);
                
                // 连接服务器
                // 发布消息
                let publish_result = client.publish("test/topic", QoS::AtMostOnce, false, "test payload").await;
                assert!(publish_result.is_ok());
                
                // 等待消息发布完成
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                
                // 关闭连接
                drop(client);
            });
        });
    });
    
}
