// testing flot.rs
extern crate flot;
use flot::{Page,Corner,MapTuple};

fn main() {
    let line_data = vec![(0.0,1.0),(1.0,4.5)];
    let points_data = vec![(0.5,1.2),(0.8,4.0)];
    //let bar_data = vec![(0.0,10.0),(1.0,5.0),(2.0,15.0)];

    let page = Page::new();
    let p = page.plot();
    p.legend_pos(Corner::None)
        .extra_symbols()        
        .xaxis()
          .bounds(Some(-0.2),Some(1.3));
          
    p.markings().horizontal_line(0.5).color("red");

    p.lines("lines",line_data).fill(0.3).line_width(0);
    p.points("points",points_data).symbol("cross").radius(10).line_width(0);
    //page.plot().bars("bars",bar_data);

    let cs = page.plot();
    cs.markings().vertical_line(4.5).color("black");
    
    let xvalues: Vec<_> = flot::range(0.0,8.0,0.2).collect();    
    cs.lines("sin",xvalues.iter().map(|&x| (x, x.sin())));
    cs.lines("cos",xvalues.as_slice().map(|x| x.cos())).color("green");
    //cs.lines("cos",flot::map(&xvalues,|x| x.cos()));
    
    let xdata = [1,2,5];
    let ydata = vec![0.5,1.0,0.5];
    cs.points("data",flot::zip(&xdata,&ydata));


    page.render("test.html").expect("i/o error");
}
