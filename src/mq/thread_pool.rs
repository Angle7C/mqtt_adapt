use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use std::sync::Arc;
use crate::mq::message::MqMessage;

/// MQ任务类型
type MqTask = Box<dyn FnOnce() -> Result<()> + Send + Sync + 'static>;

/// MQ线程池服务
pub struct MqThreadPool {
    /// 任务发送器
    task_sender: Option<mpsc::Sender<MqTask>>,
    /// 工作线程句柄
    workers: Vec<JoinHandle<()>>,
}

impl MqThreadPool {
    /// 创建新的MQ线程池
    /// 
    /// # 参数
    /// - `size`: 线程池大小
    /// 
    /// # 示例
    /// ```
    /// // 创建一个包含4个线程的线程池
    /// ```
    pub async fn new(size: usize) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<MqTask>(100);
        let rx = Arc::new(tokio::sync::Mutex::new(rx));
        
        let mut workers = Vec::with_capacity(size);
        
        // 创建工作线程
        for i in 0..size {
            let rx_clone = rx.clone();
            
            let worker = tokio::spawn(async move {
                let worker_id = i;
                
                loop {
                    let mut rx_guard = rx_clone.lock().await;
                    match rx_guard.recv().await {
                        Some(task) => {
                            drop(rx_guard); // 释放锁，以便其他线程可以接收任务
                            log::debug!("Worker {} processing task", worker_id);
                            if let Err(e) = task() {
                                log::error!("Worker {} failed to process task: {:?}", worker_id, e);
                            }
                        }
                        None => {
                            // 通道关闭，退出循环
                            log::debug!("Worker {} exiting", worker_id);
                            break;
                        }
                    }
                }
            });
            
            workers.push(worker);
        }
        
        Ok(Self {
            task_sender: Some(tx),
            workers,
        })
    }
    
    /// 提交任务到线程池
    pub async fn submit<F>(&self, task: F) -> Result<()>
    where
        F: FnOnce() -> Result<()> + Send + Sync + 'static,
    {
        if let Some(sender) = &self.task_sender {
            sender.send(Box::new(task)).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("ThreadPool is shutdown"))
        }
    }
    
    /// 提交消息处理任务
    pub async fn submit_message_handler<F>(&self, message: MqMessage, handler: F) -> Result<()>
    where
        F: Fn(MqMessage) -> Result<()> + Send + Sync + 'static,
    {
        self.submit(move || handler(message)).await
    }
    
    /// 关闭线程池
    pub async fn shutdown(&mut self) {
        // 关闭任务通道
        self.task_sender.take();
        
        // 等待所有工作线程完成
        for worker in self.workers.drain(..) {
            let _ = worker.await;
        }
    }
    
    /// 获取线程池大小
    pub fn size(&self) -> usize {
        self.workers.len()
    }
}

/// 默认MQ线程池大小
pub const DEFAULT_MQ_THREAD_POOL_SIZE: usize = 4;

/// 创建默认大小的MQ线程池
pub async fn create_default_mq_thread_pool() -> Result<MqThreadPool> {
    MqThreadPool::new(DEFAULT_MQ_THREAD_POOL_SIZE).await
}
