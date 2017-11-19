extern crate flot;

fn main() {
    let page = flot::Page::new("");

    let p = page.plot("");
    p.yaxis().transform("Math.log(v+0.0001)").min(0.0).tick_values(&[0.1,1.0,10.10,100.0,1000.0]);

    p.lines("",flot::mapv(flot::range(0.1,5.0,0.05),|x| x.exp()));
    page.render("log-axis.html").unwrap();
}
