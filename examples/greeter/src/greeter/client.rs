/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

pub mod protos {
    #![allow(non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/org.apache.dubbo.sample.tri.rs"));
}

use std::env;

use dubbo::codegen::*;

use dubbo_base::Url;
use futures_util::StreamExt;
use dubbo_config::RootConfig;
use protos::{greeter_client::GreeterClient, GreeterRequest};
use registry_nacos::NacosRegistry;
use registry_zookeeper::ZookeeperRegistry;

#[tokio::main]
async fn main() {
    dubbo_logger::init();

    let mut builder = ClientBuilder::new();

    let mut url = Url::from_url(&format!(
        "provider://{}:{}/{}",
        "127.0.0.1", "1234", "phoenixakacenter.PhoenixAkaCenter"
    ))
        .unwrap();
    url.set_param("anyhost", "true");
    url.set_param("application", "phoenixakacenter-provider");
    url.set_param("background", "false");
    url.set_param("bind.ip", "127.0.0.1");
    url.set_param("bind.port", "1234");
    url.set_param("category", "configurators");
    url.set_param("check", "false");
    url.set_param("deprecated", "false");
    url.set_param("dubbo", "2.0.2");
    url.set_param("dynamic", "true");
    url.set_param("generic", "false");
    url.set_param("interface", "phoenixakacenter.PhoenixAkaCenter");
    url.set_param("ipv6", "fd00:6cb1:58a2:8ddf:0:0:0:1000");
    url.set_param("methods", "query_exchange_rate");
    url.set_param("pid", "44270");
    url.set_param("service-name-mapping", "true");
    url.set_param("side", "provider");

    if let Ok(zk_servers) = env::var("ZOOKEEPER_SERVERS") {
        let zkr = ZookeeperRegistry::new(&zk_servers);
        let directory = RegistryDirectory::new(url, Box::new(zkr));
        builder = builder.with_directory(Box::new(directory));
    } else if let Ok(nacos_url_str) = env::var("NACOS_URL") {
        // NACOS_URL=nacos://mse-96efa264-p.nacos-ans.mse.aliyuncs.com
        let nacos_url = Url::from_url(&nacos_url_str).unwrap();
        let registry = NacosRegistry::new(nacos_url);
        let directory = RegistryDirectory::new(url, Box::new(registry));
        builder = builder.with_directory(Box::new(directory));
    } else {
        builder = builder.with_host("http://127.0.0.1:8888");
    }

    let mut cli = GreeterClient::new(builder);

    println!("# unary call");
    let resp = cli
        .greet(Request::new(GreeterRequest {
            name: "message from client".to_string(),
        }))
        .await;
    let resp = match resp {
        Ok(resp) => resp,
        Err(err) => return println!("{:?}", err),
    };
    let (_parts, body) = resp.into_parts();
    println!("Response: {:?}", body);

    println!("# client stream");
    let data = vec![
        GreeterRequest {
            name: "msg1 from client streaming".to_string(),
        },
        GreeterRequest {
            name: "msg2 from client streaming".to_string(),
        },
        GreeterRequest {
            name: "msg3 from client streaming".to_string(),
        },
    ];
    let req = futures_util::stream::iter(data);
    let resp = cli.greet_client_stream(req).await;
    let client_streaming_resp = match resp {
        Ok(resp) => resp,
        Err(err) => return println!("{:?}", err),
    };
    let (_parts, resp_body) = client_streaming_resp.into_parts();
    println!("client streaming, Response: {:?}", resp_body);

    println!("# bi stream");
    let data = vec![
        GreeterRequest {
            name: "msg1 from client".to_string(),
        },
        GreeterRequest {
            name: "msg2 from client".to_string(),
        },
        GreeterRequest {
            name: "msg3 from client".to_string(),
        },
    ];
    let req = futures_util::stream::iter(data);

    let bidi_resp = cli.greet_stream(req).await.unwrap();

    let (parts, mut body) = bidi_resp.into_parts();
    println!("parts: {:?}", parts);
    while let Some(item) = body.next().await {
        match item {
            Ok(v) => {
                println!("reply: {:?}", v);
            }
            Err(err) => {
                println!("err: {:?}", err);
            }
        }
    }
    let trailer = body.trailer().await.unwrap();
    println!("trailer: {:?}", trailer);

    println!("# server stream");
    let resp = cli
        .greet_server_stream(Request::new(GreeterRequest {
            name: "server streaming req".to_string(),
        }))
        .await
        .unwrap();

    let (parts, mut body) = resp.into_parts();
    println!("parts: {:?}", parts);
    while let Some(item) = body.next().await {
        match item {
            Ok(v) => {
                println!("reply: {:?}", v);
            }
            Err(err) => {
                println!("err: {:?}", err);
            }
        }
    }
    let trailer = body.trailer().await.unwrap();
    println!("trailer: {:?}", trailer);
}
