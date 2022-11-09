use ethers::types::H160;

// inspired from https://github.com/Ethnical/SmartConStream/blob/main/src/main.rs#L165
#[allow(unused_variables)]
#[allow(dead_code)]
fn get_source_code(address: &H160) -> Vec<(String, Vec<String>)> {
    let mut result: Vec<(String, Vec<String>)> = vec![];
    let link = format!("https://etherscan.io/contractdiffchecker?a1={:?}", address);
    let response = reqwest::blocking::get(link).unwrap().text().unwrap();
    let document = scraper::Html::parse_document(&response);

    // Initialize the first file selector
    let mut i = 1;
    let file_name_selector = scraper::Selector::parse("span#fileName_11").unwrap();
    let source_code_selector = scraper::Selector::parse("pre#sourceCode_11").unwrap();
    let mut fn_select = document.select(&file_name_selector).next();
    let mut sc_select = document.select(&source_code_selector).next();

    // Loop through all the file selectors until there is none
    while fn_select.is_some() && sc_select.is_some() {
        let smart_contract = sc_select
            .unwrap()
            .inner_html()
            .split('\n')
            .map(|s| s.to_owned())
            .collect();
        result.push((fn_select.unwrap().inner_html(), smart_contract));
        i += 1;
        let file_selector = format!("span#fileName_1{}", i);
        let source_selector = format!("pre#sourceCode_1{}", i);
        let file_name_selector = scraper::Selector::parse(&file_selector).unwrap();
        let source_code_selector = scraper::Selector::parse(&source_selector).unwrap();
        fn_select = document.select(&file_name_selector).next();
        sc_select = document.select(&source_code_selector).next();
    }
    result
}

mod test {
    use ethers::types::H160;
    use std::str::FromStr;

    #[test]
    fn test() {
        let address = H160::from_str("0xac046563e7104292fe9130b08360049f79a3b5bf").unwrap();
        let result = super::get_source_code(&address);
        assert!(result[0].0 == "OfficalWelcomeBackTrump.sol".to_owned());
    }
}
