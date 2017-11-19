//! [Flot](http://www.flotcharts.org/) is a JavaScript library for generating
//! attractive data plots.  Although usually used to enhance interactive
//! websites, **flot-rs** is a nice way for command-line programs to create
//! standalone HTML documents with plots. By default these refer to online
//! sources, so they can be handed over to anybody else for display.
//!
//! ```
//! extern crate flot;
//!
//! fn main() {
//!     let line_data = vec![(0.0,1.0),(1.0,4.5)];
//!     let points_data = vec![(0.5,1.2),(0.8,4.0)];
//!
//!     let page = flot::Page::new("");
//!
//!     let p = page.plot("Lines and Points");
//!     p.lines("lines",line_data).fill(0.3).line_width(0);
//!     p.points("points",points_data).symbol("circle");
//!
//!     page.render("simple.html").expect("i/o error");
//! }
//!
//! ```
//!
//! A `Page` may contain multiple plots; plots may contain multiple
//! series with chart types (`lines`,`points`,`bars`).
//!
//! The result of running this program is to create 'simple.html', which
//! can be opened in your browser.
//!
//! `Page` can be given a title, which if non-empty will both set the title
//! of the document and create a H1 heading. Likewise, the `plot` method is
//! given a title which if non-empty will provide a centered H2 heading for
//! the plot.
//!
//! ## Ways of specifying Data
//!
//! By default, the series constructors take anything that converts to an
//! iterator of `(f64,f64)` x-y pairs.
//! Note that the vectors `line_data` and `points_data` are consumed
//! by these calls.
//!
//! If you have a source of tuples that isn't `(f64,f64)`, then
//! `flot::to_f64` will convert that into a form that _flot-rs_ accepts, provided
//! that those types convert cleanly into `f64`.
//!
//! Alternatively, you can map a iterator of references with a function - `flot::mapr`
//! produces the desired points iterator, which here we collect into a vector.
//!
//! ```
//! extern crate flot;
//!
//! fn make_gaussian(xvalues: &[f64], m: f64, s: f64) -> Vec<(f64,f64)> {
//!     use std::f64::consts::PI;
//!     let s2 = 2.0*s*s;
//!     let norm = 1.0/(s2*PI).sqrt();
//!     flot::mapr (
//!         xvalues,
//!         move |x| norm*(-(x-m).powi(2)/s2).exp()
//!     ).collect()
//! }
//!
//! fn main() {
//!     let page = flot::Page::new("");
//!
//!     let p = page.plot("Normal distribution").size(500,300);
//!     let xvalues: Vec<_> = flot::range(0.0,10.0,0.1).collect();
//!     p.lines("norm σ=1.0",make_gaussian(&xvalues,5.0,1.0));
//!     p.lines("norm σ=0.7",make_gaussian(&xvalues,6.0,0.5));
//!
//!     page.render("normal.html").unwrap();
//! }
//! ```
//! `range` is a little convenience iterator for making ranges of floating-point
//! values (subsequently I've discovered that the [itertools-num](https://docs.rs/itertools-num)
//! crate provides something similar - see `linspace`).
//!
//!
//! `flot::mapv` is similar, except it takes an iterator of _values_. Here are the
//! squares of all integers from 0 to 9:
//!
//! ```rust,ignore
//!     page.plot().legend_pos(Corner::TopLeft)
//!         .bars("squares",mapv(0..10,|x| x*x))
//!         .width(0.75);
//! ```
//! (The iterator given to `mapr` and `mapv` can provide any values which can be
//! _converted_ into a `f64`, so the integer range works.)
//!
//! Finally, `flot::zip` can take two iterators of references, which are zipped
//! together into point tuples. This is useful if you have separate x and y data
//! as slices or vectors.
extern crate typed_arena;
use typed_arena::Arena;

#[macro_use]
extern crate json;
use json::JsonValue;

use std::io;
use std::io::Write;

/// Iterator type for floating-point range iterator
pub struct FRange {
    val: f64,
    end: f64,
    incr: f64
}

/// generates an iterator between `x1` and `x2`, step `skip`
/// over floating point numbers.
/// Similar to `linspace` in the **itertools-num** crate
pub fn range(x1: f64, x2: f64, skip: f64) -> FRange {
    FRange {val: x1, end: x2, incr: skip}
}

impl Iterator for FRange {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.val;
        if res >= self.end {
            None
        } else {
            self.val += self.incr;
            Some(res)
        }
    }
}

/// join two iterators of references together to produce point tuples.
/// The reference types can be anything that converts to `f64`
pub fn zip<'a,I1,I2,T1,T2>(x: I1, y: I2) -> Box<Iterator<Item=(f64,f64)>+'a>
where I1: IntoIterator<Item=&'a T1>+'a, I2: IntoIterator<Item=&'a T2>+'a,
    T1: Into<f64>+Copy+'a, T2: Into<f64>+Copy+'a
{
    Box::new(x.into_iter().zip(y).map(|(&x,&y)| (x.into(),y.into())))
}

/// take an iterator of references to tuples of two types and produce point tuples.
/// The types can be anything that converts to `f64`
pub fn to_f64 <'a,I,T1,T2>(x: I) -> Box<Iterator<Item=(f64,f64)>+'a>
where I: IntoIterator<Item=&'a (T1,T2)>+'a,
    T1: Into<f64>+Copy+'a, T2: Into<f64>+Copy+'a
{
    Box::new(x.into_iter().map(|&(x,y)| (x.into(),y.into())))
}


/// data from an iterator of references plotted against index.
/// Like `zip`, the reference type can be anything that converts to `f64`
pub fn valr<'a,I,T>(y: I) -> Box<Iterator<Item=(f64,f64)>+'a>
where I: IntoIterator<Item=&'a T>+'a,
    T: Into<f64>+Copy+'a
{
    Box::new((0..).into_iter().zip(y).map(|(x,&y)| (x.into(),y.into())))
}


/// map an iterator of references with a function producing point tuples.
/// Like `zip`, the reference type can be anything that converts to `f64`
pub fn mapr<'a,I,T,F>(x: I, f: F) -> Box<Iterator<Item=(f64,f64)>+'a>
where I: IntoIterator<Item=&'a T>+'a, F: Fn(f64)->f64 + 'a,
    T: Into<f64>+Copy+'a
{
    Box::new(x.into_iter().map(move |&x| { let fv = x.into(); (fv,f(fv))}))
}

/// map an iterator of values with a function producing point tuples.
/// The value type can be anything that converts to `f64`
pub fn mapv<'a,I,T,F>(x: I, f: F) -> Box<Iterator<Item=(f64,f64)>+'a>
where I: IntoIterator<Item=T>+'a, F: Fn(f64)->f64 + 'a,
    T: Into<f64>+Copy+'a
{
    Box::new(x.into_iter().map(move |x| { let fv = x.into(); (fv,f(fv))}))
}


enum PlotKind {
    Lines,
    Points,
    Bars
}

impl PlotKind {
    fn to_str(&self) -> &'static str {
        match *self {
            PlotKind::Lines => "lines",
            PlotKind::Points => "points",
            PlotKind::Bars => "bars"
        }
    }
}

/// describes a data series which can be plotted either as lines, points or bars
pub struct Series {
    data: JsonValue,
    kind: PlotKind,
    symbols: bool,
}

impl Series {
    fn new<T>(kind: PlotKind, label: &str, data: T) -> Series
    where T: IntoIterator<Item=(f64,f64)> {

        let mut arr = JsonValue::new_array();
        for p in data.into_iter() {
            arr.push(array![p.0,p.1]).unwrap();
        }
        let jlbl = if label.is_empty() {JsonValue::Null} else {label.into()};
        let mut data = object! {
            "label" => jlbl,
            "data" => arr
        };
        data[kind.to_str()] = object!{"show" => true};
        Series {data: data, kind: kind, symbols: false}
    }

    fn kind_ref(&mut self) -> &mut JsonValue {
        &mut self.data[self.kind.to_str()]
    }

    /// set the xaxis for this series (2 for second)
    pub fn xaxis(&mut self, which: u32) -> &mut Self {
        self.data["xaxis"] = which.into();
        self
    }

    /// set the yaxis for this series (2 for second)
    pub fn yaxis(&mut self, which: u32) -> &mut Self {
        self.data["yaxis"] = which.into();
        self
    }

    /// set the fill colour underneath lines or in bars as an alpha value.
    pub fn fill(&mut self, opacity: f32) -> &mut Self {
        self.kind_ref()["fill"] = opacity.into();
        self
    }

    /// set the fill colour underneath lines or in bars as an HTML colour.
    pub fn fill_color(&mut self, color: &str) -> &mut Self {
        self.kind_ref()["fillColor"] = color.into();
        self
    }

    /// set the line colour as an HTML colour.
    pub fn color(&mut self, color: &str) -> &mut Self {
        self.data["color"] = color.into();
        self
    }

    /// width of the line - zero for no shadow.
    pub fn line_width(&mut self, size: u32) -> &mut Self {
        self.kind_ref()["lineWidth"] = size.into();
        self
    }

    /// radius for points (points only)
    pub fn radius(&mut self, size: u32) -> &mut Self {
        match self.kind {
            PlotKind::Points => self.kind_ref()["radius"] = size.into(),
            _ => panic!("radius() only applies to points")
        }
        self
    }

    /// symbol for points (points only)
    pub fn symbol(&mut self, name: &str) -> &mut Self {
        match self.kind {
            PlotKind::Points => {
                self.symbols = true;
                self.kind_ref()["symbol"] = name.into();
            },
            _ => panic!("symbol() only applies to points")
        }
        self
    }

    /// draw steps between points (lines only)
    pub fn steps(&mut self) -> &mut Self {
        match self.kind {
            PlotKind::Lines => self.kind_ref()["steps"] = true.into(),
            _ => panic!("steps() only applies to lines")
        }
        self
    }

    /// set width of bars (bars only)
    pub fn width(&mut self, width: f64) -> &mut Self {
        match self.kind {
            PlotKind::Bars => self.kind_ref()["barWidth"] = width.into(),
            _ => panic!("bar_width() only applies to bars")
        }
        self
    }



}

/// describes position of legend (None for no legend)
pub enum Corner {
    None,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

impl Corner {
    fn to_str(&self) -> &'static str {
        use Corner::*;
        match *self {
            None => "none",
            TopRight => "ne",
            TopLeft => "nw",
            BottomRight => "sw",
            BottomLeft => "se",
        }
    }
}

/// describes sides of plot for axis position
pub enum Side {
    Right,
    Left,
    Bottom,
    Top,
}

impl Side {
    fn to_str(&self) -> &'static str {
        use Side::*;
        match *self {
            Right => "right",
            Left => "left",
            Bottom => "bottom",
            Top => "top",
        }
    }
}

/// represents an axis
pub struct Axis<'a> {
    which: &'static str,
    plot: &'a mut Plot,
    idx: usize,
}

const TICK_FORMAT: &str = "v.toFixed(a.tickDecimals)";

impl <'a> Axis<'a> {
    fn new(which: &'static str, plot: &'a mut Plot, idx: usize) -> Axis<'a> {
        if plot.options[which].is_null() {
            plot.options[which] = array![object!{}];
        }
        if idx > 1 {
            plot.options[which].push(object!{}).unwrap();
        }
        Axis{which: which, plot: plot, idx: idx-1}
    }

    pub fn set_option(&mut self, key: &str, val: JsonValue) -> &mut Self {
        self.plot.options[self.which][self.idx][key] = val;
        self
    }

    fn axis_function(&mut self, var: &str, fun: &str) -> &mut Self {
        self.plot.option_functions.push(format!("{}[{}].{} = {}",self.which,self.idx,var,fun));
        self
    }

    /// conversion function to apply to axis values.
    /// Either a Javascript expression in `v` or the full function.
    pub fn transform(&mut self, expr_or_fun: &str) -> &mut Self {
        let s;
        self.axis_function("transform",if expr_or_fun.starts_with("function") {
            expr_or_fun
        } else {
            s = format!("function (v) {{ return {}; }}",expr_or_fun);
            &s
        })
    }

    pub fn label_formatter(&mut self, fun: &str) -> &mut Self {
        self.axis_function("tickFormatter",fun)
    }

    /// append a string to the default label
    pub fn label_post(&mut self, s: &str) -> &mut Self {
        self.label_formatter(
            &format!("function(v,a) {{ return {} + {:?}; }}",TICK_FORMAT,s)
        )
    }

    /// prepend a string to the default label
    pub fn label_pre(&mut self, s: &str) -> &mut Self {
        self.label_formatter(
            &format!("function(v,a) {{ return {:?} + {}; }}",s,TICK_FORMAT)
        )
    }

    /// force minimum value on axis
    pub fn min(&mut self, min: f64) -> &mut Self {
        self.set_option("min",min.into());
        self
    }

    /// force maximum value on axis
    pub fn max(&mut self, max: f64) -> &mut Self {
        self.set_option("max",max.into());
        self
    }

    /// set the position of an axis
    pub fn position(&mut self, side: Side) -> &mut Self {
        let pos = side.to_str();
        // TBD also multiple x!
        if pos == "right" {
            self.set_option("alignTicksWithAxis",1.into());
        }
        self.set_option("position",pos.into())
    }

    /// indicates that this axis uses time values.
    /// These must be specified in milliseconds
    pub fn time(&mut self) -> &mut Self {
        self.plot.time = true;
        self.set_option("mode","time".into())
    }

    /// explicitly provide tick values.
    pub fn tick_values(&mut self, vv: &[f64]) -> &mut Self {
        let mut arr = JsonValue::new_array();
        for v in vv {
            arr.push(JsonValue::from(*v)).unwrap();
        }
        self.set_option("ticks",arr)
    }

    /// explicitly provide tick values and labels.
    pub fn tick_values_and_labels(&mut self, vv: &[(f64,&str)]) -> &mut Self {
        let mut arr = JsonValue::new_array();
        for p in vv {
            arr.push(array![p.0,p.1]).unwrap();
        }
        self.set_option("ticks",arr)
    }

}

/// represents 'markings' or plot annotations.
pub struct Markings<'a> {
    plot: &'a mut Plot,
}

impl <'a> Markings<'a> {
    fn new(plot: &'a mut Plot) -> Markings<'a> {
        plot.set_option("grid","markings",array![]);
        Markings{plot: plot}
    }

    fn markings(&mut self) -> &mut JsonValue {
        &mut self.plot.options["grid"]["markings"]
    }

    pub fn add_marking(&mut self, val: JsonValue) -> &mut Self {
        self.markings().push(val).unwrap(); // it must be an array
        self
    }

    /// vertical band over plot
    pub fn vertical_area(&mut self, p1: f64, p2: f64) -> &mut Self {
        self.add_marking(object!{"xaxis" => object!{"from"=>p1,"to"=>p2 } })
    }

    /// horizontal band over plot
    pub fn horizontal_area(&mut self, p1: f64, p2: f64) -> &mut Self {
        self.add_marking(object!{"yaxis" => object!{"from"=>p1,"to"=>p2 } })
    }

    /// vertical line at x position
    pub fn vertical_line(&mut self, pos: f64) -> &mut Self {
        self.vertical_area(pos,pos)
    }

    /// horizontal line at y position
    pub fn horizontal_line(&mut self, pos: f64) -> &mut Self {
        self.horizontal_area(pos,pos)
    }

    /// rectangular area specified as (x1,x2,y1,y2)
    pub fn area(&mut self, x1: f64, x2: f64, y1: f64, y2: f64) -> &mut Self {
        self.add_marking(object!{
            "xaxis" => object!{"from"=>x1,"to"=>x2 },
            "yaxis" => object!{"from"=>y1,"to"=>y2 }
        })
    }

    /// set the color of the last marking defined
    pub fn color(&mut self, color: &str) -> &mut Self {
        {
            let mut arr = self.markings();
            let len = arr.len();
            arr[len-1]["color"] = color.into();
        }
        self
    }

}

/// represents the grid area of the plot
pub struct Grid<'a> {
    plot: &'a mut Plot,
}

impl <'a> Grid<'a> {
    fn new(plot: &'a mut Plot) -> Grid<'a> {
        if plot.options["grid"].is_null() {
            plot.options["grid"] = object!{};
        }
        Grid{plot: plot}
    }

    /// set any grid option not covered by this API
    /// https://github.com/flot/flot/blob/master/API.md#customizing-the-grid
    pub fn set_option(&mut self, key: &str, val: JsonValue) -> &mut Self {
        self.plot.options["grid"][key] = val;
        self
    }

    /// hide the grid completely
    pub fn hide(&mut self) -> &mut Self {
        self.set_option("show",false.into())
    }

    /// foreground colour
    pub fn color(&mut self, front: &str) -> &mut Self {
        self.set_option("color",front.into())
    }

    /// background colour
    pub fn background_color(&mut self, back: &str) -> &mut Self {
        self.set_option("backgroundColor",back.into())
    }

    /// background gradient, from bottom colour to top colour
    pub fn background_gradient(&mut self, bottom: &str, top: &str) -> &mut Self {
        self.set_option("backgroundColor",object!{"colors" => array![bottom,top]})
    }

}

/// represents the legend of the plot
pub struct Legend<'a> {
    plot: &'a mut Plot,
}

impl <'a> Legend<'a> {
    fn new(plot: &'a mut Plot) -> Legend<'a> {
        if plot.options["legend"].is_null() {
            plot.options["legend"] = object!{};
        }
        Legend{plot: plot}
    }

    /// any legend option not covered by this API
    /// https://github.com/flot/flot/blob/master/API.md#customizing-the-legend
    pub fn set_option(&mut self, key: &str, val: JsonValue) -> &mut Self {
        self.plot.options["legend"][key] = val;
        self
    }

    /// position of legend (Corner::None to hide)
    pub fn pos(&mut self, pos: Corner) -> &mut Self {
        match pos {
            Corner::None => self.set_option("show",false.into()),
            _ => self.set_option("position",pos.to_str().into())
        }
    }

}


/// represents a particular plot
pub struct Plot {
    series: Arena<Series>,
    placeholder: String,
    options: JsonValue,
    time: bool,
    symbols: bool,
    bounds: (u32,u32),
    title: String,
    option_functions: Vec<String>,
    description: Vec<String>,
}

impl Plot {
    fn new(name: &str, title: &str, bounds: (u32,u32)) -> Plot {
        Plot {
            series: Arena::new(),
            placeholder: name.into(),
            options: object!{},
            time: false,
            symbols: false,
            bounds: bounds,
            title: title.into(),
            option_functions: Vec::new(),
            description: Vec::new(),
        }
    }

    /// add a paragrath of text below a plot.
    pub fn text(&mut self, txt: &str) -> &mut Self {
        let mut escaped = String::new();
        for ch in txt.chars() {
            match ch {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            _ => escaped.push(ch)
            }
        }
        self.description.push(escaped);
        self
    }

    /// add a paragrath of HTML below a plot.
    pub fn html(&mut self, txt: &str) -> &mut Self {
        self.description.push(txt.into());
        self
    }


    /// the size in pixels (width,height) of the plot area
    pub fn size(&mut self,width:u32,height:u32) -> &mut Self {
        self.bounds = (width,height);
        self
    }

    /// x axis object
    pub fn xaxis<'a>(&'a mut self) -> Axis<'a> {
        Axis::new("xaxes",self,1)
    }

    /// y axis object
    pub fn yaxis<'a>(&'a mut self) -> Axis<'a> {
        Axis::new("yaxes",self,1)
    }

    /// second y axis object
    pub fn yaxis2<'a>(&'a mut self) -> Axis<'a> {
        Axis::new("yaxes",self,2)
    }

    /// create a data series with individual points.
    /// The data is anything that converts to an iterator
    /// of `(f64,f64)` tuples. If label is the empty string,
    /// don't show in legend
    pub fn points<T>(&self, label: &str, data: T) -> &mut Series
    where T: IntoIterator<Item=(f64,f64)> {
        self.series.alloc(Series::new(PlotKind::Points,label,data))
    }

    /// create a data series joined with lines.
    pub fn lines<T>(&self, label: &str, data: T) -> &mut Series
    where T: IntoIterator<Item=(f64,f64)> {
        self.series.alloc(Series::new(PlotKind::Lines,label,data))
    }

    /// create a data series with bars (histogram).
    pub fn bars<T>(&self, label: &str, data: T) -> &mut Series
    where T: IntoIterator<Item=(f64,f64)> {
        self.series.alloc(Series::new(PlotKind::Bars,label,data))
    }

    /// position of legend (Corner::None to hide)
    pub fn legend_pos(&mut self, pos: Corner) -> &mut Self {
        self.legend().pos(pos);
        self
    }

    /// legend object
    pub fn legend<'a>(&'a mut self) -> Legend<'a> {
        Legend::new(self)
    }

    /// grid object
    pub fn grid<'a>(&'a mut self) -> Grid<'a> {
        Grid::new(self)
    }

    pub fn extra_symbols(&mut self) -> &mut Self {
        self.symbols = true;
        self
    }

    /// object to create markings like lines and areas
    pub fn markings<'a>(&'a mut self) -> Markings<'a> {
        Markings::new(self)
    }

    /// set any option field not exposed in this API.
    pub fn set_option(&mut self, key: &str, subkey: &str, val: JsonValue) -> &mut Self {
        if self.options[key].is_null() {
            self.options[key] = object!{};
        }
        self.options[key][subkey] = val;
        self
    }

    fn render_placeholder(&self, f: &mut Write) -> io::Result<()> {
        if ! self.title.is_empty() {
            write!(f, "<h2 style='text-align: center;width:{}px'>{}</h2>\n"
                ,self.bounds.0,self.title)?;
        }
        write!(f, "<div id={:?} style=\"width:{}px;height:{}px\"></div>\n",
            self.placeholder,self.bounds.0,self.bounds.1)?;

        for s in &self.description {
            write!(f, "<p style='width:{}px;margin-left:2em;margin-right:2em'>{}</p>",self.bounds.0,s)?;
        }
        Ok(())
    }

    fn render_script(self, f: &mut Write) -> io::Result<()> {
        let series = self.series.into_vec();
        let mut data = '['.to_string();
        let basename = "plot";
        let mut k = 1;
        for s in &series {
            let varname = format!("{}_{}",basename,k);
            k += 1;
            write!(f,"var {} = {};\n",varname,s.data)?;
            data += &varname;
            data.push(',');
        }
        data.pop();
        data.push(']');
        let option_var = format!("{}_options",basename);
        write!(f,"var {} = {};\n",option_var,self.options)?;
        for lf in &self.option_functions {
            write!(f,"{}.{};\n",option_var,lf)?;
        }
        write!(f,"$.plot($(\"#{}\"),{},{});\n",self.placeholder,data,option_var)
    }

}

use std::env;
use std::fs::File;
use std::mem;
use std::cell::Cell;

/// represents an HTML document containing plots
pub struct Page {
    plots: Arena<Plot>,
    count: Cell<usize>,
    title: String,
    bounds: (u32,u32),
}

fn script(base: &str, name: &str) -> String {
    format!("<script language=\"javascript\" type=\"text/javascript\" src=\"{}/{}\"></script>",
        base,name)
}

impl Page {
    /// create the page.
    /// If the title isn't empty then
    /// add a document header (H1) and set title
    pub fn new(title: &str) -> Page {
        Page {
            plots: Arena::new(),
            count: Cell::new(0),
            title: title.into(),
            bounds: (800,300),
        }
    }

    /// create a new plot.
    /// If the title isn't empty, then
    /// create a header (centered H2) for the plot
    pub fn plot(&self, title: &str) -> &mut Plot {
        let count = &self.count;
        count.set(count.get() + 1);
        let name = format!("plot{}",self.count.get());
        self.plots.alloc(Plot::new(&name,title,self.bounds))
    }

    /// the size in pixels (width,height) of _all_ the plots.
    /// Can be overriden with the `size` method of indivdiual plots.
    pub fn size(&mut self,width:u32,height:u32) -> &mut Self {
        self.bounds = (width,height);
        self
    }


    /// render the page as HTML to the given file.
    /// Warning: this must absolutely be the last call when
    /// creating Flot plots - any attempt to access plot
    /// objects after this will lead to tears.
    pub fn render(&self, file: &str) -> io::Result<()> {
        // this is deeply dubious. In an ideal world with non-lexical lifetimes,
        // this could be a self method, since it is _only_ called after all
        // the plots have been defined. It cannot be &mut self, because borrows
        // are already happening, so we do a &Self -> &mut Self conversion.
        //
        // In an even more ideal world, typed_arena::Arena would have an
        // iterator over references
        let mut nplots = Arena::new();
        let self_ptr: *const Self = self;
        let mut_self: &mut Self = unsafe { &mut * (self_ptr as *mut Self) };
        mem::swap(&mut mut_self.plots, &mut nplots);

        let plots = nplots.into_vec();

        let (jquery,flot) = if let Ok(f) = env::var("FLOT") {
            let local = format!("file://{}",f);
            (local.clone(),local.clone())
        } else {
            (
                "https://cdnjs.cloudflare.com/ajax/libs/jquery/3.2.1".to_string(),
                "https://cdnjs.cloudflare.com/ajax/libs/flot/0.8.3".to_string()
            )
        };
        let mut f = File::create(file)?;
        let header = format!("
<html>
 <head>
    <meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\">
    <title>{}</title>
", if ! self.title.is_empty() {&self.title} else {"Flot"});
        write!(f,"{}{}\n{}\n",header,
            script(&jquery,"jquery.min.js"),
            script(&flot,"jquery.flot.min.js"))?;
        if plots.iter().any(|p| p.time) {
            write!(f,"{}\n",script(&flot,"jquery.flot.time.min.js"))?;
        }
        if plots.iter().any(|p| p.symbols) {
            write!(f,"{}\n",script(&flot,"jquery.flot.symbol.min.js"))?;
        }
        write!(f,"</head>\n</body>\n")?;
        if ! self.title.is_empty() {
            write!(f,"<h1>{}</h1>\n",self.title)?;
        }
        for p in &plots {
            p.render_placeholder(&mut f)?;
        }
        write!(f,"<script type=\"text/javascript\">\n$(function () {{\n")?;
        for p in plots {
            p.render_script(&mut f)?;
        }
        write!(f,"}});\n</script>\n</body>\n</html>\n")
    }
}

