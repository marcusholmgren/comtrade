use crate::parser::CFF_HEADER_REGEXP;
use crate::{ComtradeParser, DataFormat, FileType, ParseError, ParseResult};
use std::io::BufRead;
use std::str::FromStr;

impl<T: BufRead> ComtradeParser<T> {
    pub(super) fn load_cff(&mut self) -> ParseResult<()> {
        let file = match &mut self.cff_file {
            Some(reader) => reader,
            None => {
                return Err(ParseError::new(
                    "tried to parse .cff file, but file not specified".to_string(),
                ))
            }
        };

        let mut cfg_lines: Vec<String> = vec![];
        let mut dat_lines: Vec<String> = vec![];
        let mut hdr_lines: Vec<String> = vec![];
        let mut inf_lines: Vec<String> = vec![];

        let mut current_file: Option<FileType> = None;
        let mut data_format: Option<DataFormat> = None;
        let mut data_size: Option<usize> = None;

        loop {
            let mut line = String::new();
            let bytes_read = file
                .read_line(&mut line)
                .map_err(|e| ParseError::new(format!("I/O error reading .cff file: {}", e)))?;
            if bytes_read == 0 {
                break;
            }

            // A trim() without allocations is needed for CFF_HEADER_REGEXP match,
            // but we must be careful with keeping the original line if it's pushed later.
            let trimmed_line = line.trim();

            let maybe_file_header_match = CFF_HEADER_REGEXP.captures(trimmed_line);
            if let Some(header_match) = maybe_file_header_match {
                let file_type_token = header_match.name("file_type").ok_or_else(|| {
                    ParseError::new("unable to find file type in CFF header Regexp".to_string())
                })?;

                let maybe_data_format_token = header_match.name("data_format");
                let maybe_data_size_token = header_match.name("data_size");

                current_file = Some(FileType::from_str(file_type_token.as_str())?);

                if let Some(data_format_token) = maybe_data_format_token {
                    data_format = Some(DataFormat::from_str(data_format_token.as_str())?);
                }

                if let Some(data_size_token) = maybe_data_size_token {
                    data_size = Some(data_size_token.as_str().parse::<usize>().map_err(|_| {
                        ParseError::new(format!(
                            "unable to parse .dat size: '{}'",
                            data_size_token.as_str()
                        ))
                    })?)
                }

                if current_file == Some(FileType::Dat) && data_format != Some(DataFormat::Ascii) {
                    if let Some(size) = data_size {
                        let mut buf = vec![0u8; size];
                        file.read_exact(&mut buf).map_err(|e| {
                            ParseError::new(format!("I/O error reading binary data: {}", e))
                        })?;
                        self.binary_dat_contents = buf;
                        current_file = None; // Reset to avoid appending remaining newlines
                    } else {
                        return Err(ParseError::new(
                            "binary data in .cff files must specify data size".to_string(),
                        ));
                    }
                }

                continue;
            }

            let line = line.trim().to_string();

            match current_file {
                Some(FileType::Cfg) => cfg_lines.push(line),
                Some(FileType::Dat) => {
                    // Ascii format is handled line-by-line
                    if data_format == Some(DataFormat::Ascii) {
                        dat_lines.push(line);
                    } else {
                        // Binary is handled above right after parsing the header
                        unreachable!()
                    }
                }
                Some(FileType::Hdr) => hdr_lines.push(line),
                Some(FileType::Inf) => inf_lines.push(line),
                None => {
                    return Err(ParseError::new(
                        "encountered file contents line before header in .cff".to_string(),
                    ))
                }
            }
        }

        // TODO: Create `io::Cursor()` here instead of simply whacking all the contents
        //  into a string. This would allow for buffered reading of separate files, at least.

        self.cfg_contents = cfg_lines.join("\n");
        self.ascii_dat_contents = dat_lines.join("\n");
        self.hdr_contents = hdr_lines.join("\n");
        self.inf_contents = inf_lines.join("\n");

        Ok(())
    }
}
