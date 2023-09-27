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

use std::{
    collections::HashMap,
    fmt::Debug,
    str::FromStr,
    sync::{Arc, RwLock},
};

use crate::{
    codegen::TripleInvoker,
    invocation::{Invocation, RpcInvocation},
    protocol::BoxInvoker,
    registry::{memory_registry::MemoryNotifyListener, BoxRegistry},
};
use dubbo_base::Url;
use dubbo_logger::tracing;
use dubbo_logger::tracing::log;
use dubbo_logger::tracing::log::log;

use crate::cluster::Directory;

/// Directory.
///
/// [Directory Service](http://en.wikipedia.org/wiki/Directory_service)

#[derive(Debug, Clone)]
pub struct StaticDirectory {
    uri: http::Uri,
}

impl StaticDirectory {
    pub fn new(host: &str) -> StaticDirectory {
        let uri = match http::Uri::from_str(host) {
            Ok(v) => v,
            Err(err) => {
                tracing::error!("http uri parse error: {}, host: {}", err, host);
                panic!("http uri parse error: {}, host: {}", err, host)
            }
        };
        StaticDirectory { uri: uri }
    }

    pub fn from_uri(uri: &http::Uri) -> StaticDirectory {
        StaticDirectory { uri: uri.clone() }
    }
}

impl Directory for StaticDirectory {
    fn list(&self, invocation: Arc<RpcInvocation>) -> Vec<BoxInvoker> {
        let url = Url::from_url(&format!(
            "{}://{}:{}/{}",
            self.uri.scheme_str().unwrap_or("tri"),
            self.uri.host().unwrap(),
            self.uri.port().unwrap(),
            invocation.get_target_service_unique_name(),
        ))
        .unwrap();
        let invoker = Box::new(TripleInvoker::new(url));
        vec![invoker]
    }
}

#[derive(Debug, Clone)]
pub struct RegistryDirectory {
    registry: Arc<BoxRegistry>,
    service_instances: Arc<RwLock<HashMap<String, Vec<Url>>>>,
}

impl RegistryDirectory {
    pub fn new(registry: BoxRegistry) -> RegistryDirectory {
        RegistryDirectory {
            registry: Arc::new(registry),
            service_instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Directory for RegistryDirectory {
    fn list(&self, invocation: Arc<RpcInvocation>) -> Vec<BoxInvoker> {
        let service_name = invocation.get_target_service_unique_name();

        // let url = Url::from_url(&format!(
        //     "provider://{}:{}/{}",
        //     "172.16.1.85", "8888", service_name
        // ))
        // .unwrap();

        let subscribe_url = Url::from_url("provider://172.16.1.85:8888/phoenixakacenter.PhoenixAkaCenter?anyhost=true&application=phoenixakacenter-provider&background=false&bind.ip=172.16.1.85&bind.port=8888&category=configurators&check=false&deprecated=false&dubbo=2.0.2&dynamic=true&generic=false&interface=phoenixakacenter.PhoenixAkaCenter&ipv6=fd00:6cb1:58a2:8ddf:0:0:0:1000&methods=query_exchange_rate&pid=44270&service-name-mapping=true&side=provider").unwrap();

        self.registry
            .subscribe(
                subscribe_url,
                Arc::new(MemoryNotifyListener {
                    service_instances: Arc::clone(&self.service_instances),
                }),
            )
            .expect("subscribe");

        // let binding = Vec::new();
        // let url_vec = map.get(&service_name).unwrap_or(&binding);
        let url_vec = loop {
            let map = self
                .service_instances
                .read()
                .expect("service_instances.read");

            if let Some(url_vec) = map.get("providers:phoenixakacenter.PhoenixAkaCenter::") {
                break url_vec.to_owned()
            }
        };
        // url_vec.to_vec()
        let mut invokers: Vec<BoxInvoker> = vec![];
        for item in url_vec.iter() {
            invokers.push(Box::new(TripleInvoker::new(item.clone())));
        }
        invokers
    }
}
