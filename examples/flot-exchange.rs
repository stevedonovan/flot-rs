extern crate flot;
use std::fs::File;
use std::io;
use std::io::prelude::*;


fn parse_f64(s: &str) -> f64 {
    s.trim().parse().expect("bad float")
}

fn read_data(name: &str) -> Vec<(f64,f64)> {
    let f = File::open(name).expect(&format!("cannot open {}",name));
    let buff = io::BufReader::new(f);
    buff.lines()
        .map(|line| line.unwrap().split(',').take(2)
            .map(parse_f64).collect::<Vec<_>>()
        )
        .map(|v| (1000.0*v[0],v[1])).collect()
}

fn main() {
    let page = flot::Page::new("");
    let p = page.plot("Oil Price vs Euro exchange rate")
        .legend_pos(flot::Corner::BottomRight)
        .size(900,400);
    p.xaxis().time();
    p.yaxis().min(0.0);
    p.yaxis2().position(flot::Side::Right).label_post("€");
    p.lines("dollar/euro exchange",read_data("exchangerates.csv")).yaxis(2);
    p.lines("oil price",read_data("oilprices.csv"));
    page.render("exchange.html").expect("i/o error");
}
