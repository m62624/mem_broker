use mem_broker::{message::*, Actor, Consumer, MemBroker};

#[actix::test]
async fn test() {
    let mut broker = MemBroker::new();

    let consumer1 = Consumer::new("consumer 1").start();
    let consumer2 = Consumer::new("consumer 2").start();

    let topic1 = broker.create_topic("topic 1").await;

    topic1
        .send(Subscribe::new("topic 1", consumer1.clone()))
        .await;
    topic1
        .send(Subscribe::new("topic 1", consumer2.clone()))
        .await;

    println!("consumers {:#?}, {:#?}", consumer1, consumer2)
}
