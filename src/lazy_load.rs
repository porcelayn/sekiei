use crate::file_ops::safely_write_file;
use css_minify::optimizations::{Level as CssLevel, Minifier as CssMinifier};
use minify_js::{Session, TopLevelMode, minify as js_minify};
use std::error::Error;
use std::path::Path;
use regex;

pub fn setup_lazy_loading(dist_static: &Path) -> Result<(), Box<dyn Error>> {
    let lazy_loading_js = r#"
document.addEventListener('DOMContentLoaded', () => {
    const lazyImages = document.querySelectorAll('img[data-src]');
    
    const lazyLoadOptions = {
        root: null,
        rootMargin: '200px 0px 0px 0px', 
        threshold: 0.1
    };
    
    const lazyLoadObserver = new IntersectionObserver((entries, observer) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const img = entry.target;
                img.src = img.dataset.src;
                
                img.onload = () => {
                    img.classList.add('loaded');
                    img.removeAttribute('data-src');
                    
                    const container = img.closest('.lazy-image-container');
                    if (container) {
                        const placeholder = container.querySelector('img.placeholder');
                        if (placeholder) {
                            placeholder.remove();
                        }
                    }
                };
                
                observer.unobserve(img);
            }
        });
    }, lazyLoadOptions);
    
    lazyImages.forEach(image => {
        lazyLoadObserver.observe(image);
    });
});
"#;

    let lazy_loading_css = r#"
.lazy-image-container {
    position: relative;
    overflow: hidden;
}

.lazy-image-container img.placeholder {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 1;
    opacity: 1;
}

.lazy-image-container img.loaded {
    filter: blur(0);
}

.lazy-image-container img.loaded + img.placeholder {
    opacity: 0;
}
"#;

    let js_session = Session::new();
    let mut minified_js = Vec::new();
    js_minify(
        &js_session,
        TopLevelMode::Global,
        lazy_loading_js.as_bytes(),
        &mut minified_js,
    ).expect("Failed to minify JS");
    safely_write_file(
        &dist_static.join("lazyload.js"),
        std::str::from_utf8(&minified_js)?,
    )?;
    let minified_css = CssMinifier::default()
        .minify(&lazy_loading_css, CssLevel::Three)?;
    safely_write_file(&dist_static.join("lazyload.css"), &minified_css)?;

    println!("Generated and minified lazyload.js and lazyload.css");
    Ok(())
}

pub fn add_lazy_loading(html: &str, compress_to_webp: bool) -> String {
        let mut modified_html = html.to_string();
        let re = regex::Regex::new(r#"<img\s+([^>]*)src="([^"]+)"([^>]*)>"#).unwrap();

        modified_html = re.replace_all(&modified_html, |caps: &regex::Captures| {
            let attrs_before = &caps[1];
            let src = &caps[2];
            let attrs_after = &caps[3];
            
            let src_path = Path::new(src);
            let file_stem = src_path.file_stem().unwrap_or_default().to_string_lossy();
            let orig_ext = src_path.extension().unwrap_or_default().to_string_lossy();
            
            let placeholder_path = if compress_to_webp {
                format!("/static/lazy/{}.webp", file_stem)
            } else {
                format!("/static/lazy/{}.{}", file_stem, orig_ext)
            };
            
            format!(
                r#"<div class="lazy-image-container">
                    <img {}src="{}" data-src="{}" loading="lazy" {}><img class="placeholder" src="{}" alt="loading...">
                </div>"#,
                attrs_before, placeholder_path, src, attrs_after, placeholder_path
            )
        }).to_string();

        modified_html
}