pub fn run_test(js: &str) -> Vec<bool> {
    // Added logging support from console from within context
    let context = rquickjs::Context::full(&rquickjs::Runtime::new().unwrap())
        .unwrap();
    context.runtime()
        .set_max_stack_size(5242880);
    context.with(|ctx| ctx.eval::<String, _>(js).unwrap())
}

pub fn ssr_str(js: &str) -> String {
    let all_js = fastn_js::all_js_with_test();
    let js = format!("{all_js}{js}");

    // Added logging support from console from within context
    let context = rquickjs::Context::full(&rquickjs::Runtime::new().unwrap())
        .unwrap();
    context.runtime()
        .set_max_stack_size(5242880);
    context.with(|ctx| ctx.eval::<String, _>(js).unwrap())
}

pub fn ssr(ast: &[fastn_js::Ast]) -> String {
    let js = ssr_raw_string("foo", fastn_js::to_js(ast, "foo").as_str());
    ssr_str(&js)
}

pub fn ssr_with_js_string(package_name: &str, js: &str) -> String {
    let js = ssr_raw_string(package_name, js);
    ssr_str(&js)
}

pub fn ssr_raw_string(package_name: &str, js: &str) -> String {
    format!("
        let __fastn_package_name__ = \"{}\";\n{}
        let main_wrapper = function(parent) {{
            let parenti0 = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Column);
            parenti0.setProperty(fastn_dom.PropertyKind.Width, fastn_dom.Resizing.FillContainer, inherited);
            parenti0.setProperty(fastn_dom.PropertyKind.Height, fastn_dom.Resizing.FillContainer, inherited);
            main(parenti0);
        }};
        fastnVirtual.ssr(main_wrapper);", package_name, js)
}

pub fn ssr_raw_string_without_test(package_name: &str, js: &str) -> String {
    let all_js = fastn_js::all_js_without_test_and_ftd_langugage_js();
    let raw_string = ssr_raw_string(package_name, js);
    format!("{all_js}{raw_string}")
}
