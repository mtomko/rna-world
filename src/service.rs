use crate::dna;
use crate::errors::RWError;

pub fn enzymes_csv(enzymes: &[dna::RestrictionEnzyme]) -> Result<String, RWError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(&["Name", "Recognition Sequence"])
        .map_err(|_| RWError::CsvError)
        .and_then(|_| {
            let r: Result<Vec<_>, _> = enzymes
                .iter()
                .map(|e| {
                    wtr.write_record(&[&e.name, &e.recognition_sequence])
                        .map_err(|_| RWError::CsvError)
                })
                .collect();
            r
        })
        .and_then(|_| wtr.into_inner().map_err(|_| RWError::CsvError))
        .and_then(|r| String::from_utf8(r).map_err(|_| RWError::CsvError))
}

pub fn restriction_sites_csv(
    sites: &[(usize, &dna::RestrictionEnzyme)],
) -> Result<String, RWError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(&["Index", "Name", "Recognition Sequence"])
        .map_err(|_| RWError::CsvError)
        .and_then(|_| {
            let r: Result<Vec<_>, _> = sites
                .iter()
                .map(|(i, e)| {
                    wtr.write_record(&[
                        i.to_string(),
                        e.name.clone(),
                        e.recognition_sequence.clone(),
                    ])
                    .map_err(|_| RWError::CsvError)
                })
                .collect();
            r
        })
        .and_then(|_| wtr.into_inner().map_err(|_| RWError::CsvError))
        .and_then(|r| String::from_utf8(r).map_err(|_| RWError::CsvError))
}
