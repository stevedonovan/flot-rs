extern crate flot;

fn make_gaussian(xvalues: &[f64], m: f64, s: f64) -> Vec<(f64,f64)> {
    use std::f64::consts::PI;
    let s2 = 2.0*s*s;
    let norm = 1.0/(s2*PI).sqrt();
    flot::mapr (
        xvalues,
        move |x| norm*(-(x-m).powi(2)/s2).exp()
    ).collect()
}

fn main() {
    let page = flot::Page::new("");

    let p = page.plot("Normal distribution")
        .size(500,300);
    p.grid().color("white").background_color("black");
    let xvalues = flot::range(0.0,10.0,0.1).collect::<Vec<_>>();
    p.lines("norm σ=1.0",make_gaussian(&xvalues,5.0,1.0));
    p.lines("norm σ=0.7",make_gaussian(&xvalues,6.0,0.5));

    page.render("normal.html").unwrap();

}
