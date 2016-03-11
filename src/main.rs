extern crate chan;
extern crate timely;
extern crate timely_communication;

use std::thread;
use timely::dataflow::*;
use timely::dataflow::operators::*;
use timely::progress::timestamp::RootTimestamp;
use timely_communication::Allocate;


fn main() {
    let (tx, rx) = chan::sync(0);

    thread::spawn(move || {
        for i in 11..20
        {
            tx.send(i);
        }
    });

    timely::execute_from_args(std::env::args(), move |computation| {
        // create a new input, and inspect its output
        let (mut input, probe) = computation.scoped(move |builder| {
            let index = builder.index();
            let (input, stream) = builder.new_input();
            let probe = stream.exchange(|x| *x as u64)
                .inspect(move |x| println!("worker {}:\thello {}", index, x))
                .probe().0;
            (input, probe)
        });

        // introduce data and watch!
        for round in 0..10 {
            input.send(round);
            input.advance_to(round + 1);
            while probe.le(&RootTimestamp::new(round)) {
                computation.step();
            }
        }
        loop {
            match rx.recv() {
                Some(i) =>  {
                    input.send(i);
                    input.advance_to(i + 1);
                    while probe.le(&RootTimestamp::new(i)) {
                        computation.step();
                    }
                },
                None => break,
            }
        }
    });
}
