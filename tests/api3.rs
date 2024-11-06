use actix::test;
use actix::Actor;
use mem_broker::broker::Broker;
use mem_broker::client;
use mem_broker::client::Client;
use mem_broker::topic::Topic;

#[actix::test]
async fn test_t_0() {
    let broker = Broker::new().start();

    let clinet1 = Client::new(broker.clone()).start();

    clinet1
        .send(client::message::NewTopic(
            Topic::new("cats").with_compaction(),
        ))
        .await;

    let r = clinet1.send(client::message::GetTopics).await;
    println!("{:?}", r);
}
