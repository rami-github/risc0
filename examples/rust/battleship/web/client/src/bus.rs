// Copyright 2022 Risc0, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{collections::HashSet, marker::PhantomData};
use yew_agent::{Worker, WorkerLink, HandlerId, Public};

pub struct EventBus<T: Clone + 'static> {
    link: WorkerLink<EventBus<T>>,
    subscribers: HashSet<HandlerId>,
    _marker: PhantomData<T>,
}

// TODO:: fix contract serialization
impl<T: Clone + 'static>Worker for EventBus<T> {
    type Reach = Public<Self>;
    type Message = ();
    type Input = T;
    type Output = T;

    fn create(link: WorkerLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
            _marker: PhantomData,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        for sub in self.subscribers.iter() {
            self.link.respond(*sub, msg.clone());
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}


