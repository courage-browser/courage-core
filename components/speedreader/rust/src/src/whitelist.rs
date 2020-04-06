use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use flate2::read::GzDecoder;

use crate::speedreader::{AttributeRewrite, RewriteRules, SpeedReaderConfig, SpeedReaderError};

const IMAGE_TARGET_WIDTH: u32 = 600;

#[derive(Serialize, Deserialize)]
pub struct Whitelist {
    map: HashMap<String, SpeedReaderConfig>,
}

impl Default for Whitelist {
    fn default() -> Self {
        Whitelist {
            map: HashMap::new(),
        }
    }
}

impl Whitelist {
    pub fn add_configuration(&mut self, config: SpeedReaderConfig) {
        self.map.insert(config.domain.clone(), config);
    }

    pub fn get_configuration(&self, domain: &str) -> Option<&SpeedReaderConfig> {
        if let Some(config) = self.map.get(domain) {
            return Some(config);
        }

        for (i, c) in domain[..domain.len() - 2].char_indices() {
            if c == '.' {
                let subdomain = &domain[i + 1..];
                let maybe_config = self.map.get(subdomain);
                if maybe_config.is_some() {
                    return maybe_config;
                }
            }
        }

        None
    }

    pub fn get_url_rules(&self) -> Vec<String> {
        self.map
            .values()
            .flat_map(|c| c.url_rules.iter().cloned())
            .collect()
    }

    pub fn serialize(&self) -> Result<Vec<u8>, SpeedReaderError> {
        let mut out = Vec::new();
        let j = serde_json::to_string(&self.map.values().collect::<Vec<&SpeedReaderConfig>>())?;
        out.extend_from_slice(j.as_bytes());
        Ok(out)
    }

    pub fn deserialize(serialized: &[u8]) -> Result<Self, SpeedReaderError> {
        let mut gz = GzDecoder::new(serialized);
        let mut s = String::new();
        let read = gz.read_to_string(&mut s);
        if read.is_err() {
            let decoded = std::str::from_utf8(serialized)?;
            s.clear();
            s.push_str(decoded);
        }
        let configurations: Vec<SpeedReaderConfig> = serde_json::from_str(&s)?;
        let mut whitelist = Whitelist::default();
        for config in configurations.into_iter() {
            whitelist.add_configuration(config)
        }
        Ok(whitelist)
    }

    pub fn load_predefined(&mut self) {
        self.add_configuration(SpeedReaderConfig {
            domain: "cnet.com".to_owned(),
            url_rules: vec![
                "||cnet.com/features/*".to_owned(),
                "||cnet.com/roadshow/reviews/*".to_owned(),
                "||cnet.com/roadshow/news/*".to_owned(),
                "||cnet.com/news/*".to_owned(),
                "||cnet.com/reviews/*".to_owned(),
                "||cnet.com/how-to/*".to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".article-main-body".to_owned(), ".hero-content".to_owned()],
                main_content_cleanup: vec![
                    "footer".to_owned(),
                    "noscript".to_owned(),
                    ".c-head_bottomWrapper".to_owned(),
                    ".c-head_share".to_owned(),
                    ".social-button-small-author".to_owned(),
                    ".clickToEnlarge".to_owned(),
                    ".gallery".to_owned(),
                    ".video".to_owned(),
                    ".svg-symbol".to_owned(),
                ],
                delazify: true,
                fix_embeds: true,
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "247sports.com".to_owned(),
            url_rules: vec!["||247sports.com/Article/".to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["section .article-cnt".to_owned()],
                main_content_cleanup: vec![".article-cnt__header > .container".to_owned()],
                fix_embeds: true,
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "abcnews.go.com".to_owned(),
            url_rules: vec![
              "||abcnews.go.com/*/story".to_owned(),
              "||abcnews.go.com/*/wireStory".to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
            main_content: vec![".Article__Wrapper".to_owned(), "body > script:not([src])".to_owned()],
            main_content_cleanup: vec![
                ".CalloutLink".to_owned(), ".Article__Footer".to_owned(), ".Article__Header .Share".to_owned(),
                ".MediaPlaceholder__Overlay".to_owned(),
                ".inlineElement > iframe".to_owned(),
                ".Screen__Reader__Text".to_owned(), ".taboola".to_owned(),
            ],
            fix_embeds: true,
            content_script: Some(r#"<script>
            document.querySelector(".FeaturedMedia figure img").src =
                JSON.parse(document.querySelector('script[type="application/ld+json"]').innerText).image.url;
            [...document.querySelectorAll(".InlineImage .Image__Wrapper img")]
                .map((e, i) => e.src =
                    __abcnews__.page.content.story.everscroll[0].inlines.filter(d => d.type === "image").map(i => i.imageSrc)[i])
            </script>"#.to_owned()),
            ..RewriteRules::default()
        })});

        self.add_configuration(SpeedReaderConfig {
            domain: "cnn.com".to_owned(),
            url_rules: vec![
                r#"/cnn.com\/(\d){4}\/(\d){2}\/(\d){2}\/.*index.html/"#.to_owned(),
                r#"||cnn.com/*/article/*/index.html"#.to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    ".pg-headline".to_owned(),
                    ".metadata".to_owned(),
                    ".media__video--thumbnail-wrapper img".to_owned(),
                    "[itemprop=\"articleBody\"]".to_owned(),
                ],
                main_content_cleanup: vec![
                    ".m-share".to_owned(),
                    ".pg-comments".to_owned(),
                    "[class*=\"outbrain\"]".to_owned(),
                    ".zn-story-bottom".to_owned(),
                    ".zn-body__read-more".to_owned(),
                ],
                fix_embeds: true,
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "nytimes.com".to_owned(),
            url_rules: vec![
                r#"/www.nytimes.com\/\d{4}\/\d{2}\/\d{2}\/([^\/]+(\/)?){2,3}\.html/"#.to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    "div.g-blocks".to_owned(),
                    "section[name=\"articleBody\"]".to_owned(),
                    "article header".to_owned(),
                ],
                main_content_cleanup: vec![
                    ".ad".to_owned(),
                    "header#story-header".to_owned(),
                    ".story-body-1 .lede.video".to_owned(),
                    ".visually-hidden".to_owned(),
                    "#newsletter-promo".to_owned(),
                    ".promo".to_owned(),
                    ".comments-button".to_owned(),
                    ".hidden".to_owned(),
                    ".comments".to_owned(),
                    ".supplemental".to_owned(),
                    ".nocontent".to_owned(),
                    ".story-footer-links".to_owned(),
                    "#sponsor-wrapper".to_owned(),
                    "[role=\"toolbar\"]".to_owned(),
                    "header > section".to_owned(),
                ],
                fix_embeds: true,
                content_script: Some(
                    r#"
        <script>
        [...document.querySelectorAll("figure[itemid]")].forEach(fig => {
            let lazy = fig.querySelector("[data-testid=\"lazyimage-container\"]");
            if (lazy) { lazy.innerHTML = "<img src='" + fig.getAttribute("itemid") + "'>" }
        });
        </script>
        "#
                    .to_owned(),
                ),
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "theguardian.com".to_owned(),
            url_rules: vec![
                r#"/theguardian.com\/.*\/(\d){4}\/\w+\/(\d){2}\/.*/"#.to_owned()
            ],
            declarative_rewrite: Some(RewriteRules {
            main_content: vec![
                "article header".to_owned(), ".content__article-body".to_owned(),
            ],
            main_content_cleanup: vec![
                ".hide-on-mobile".to_owned(), ".inline-icon".to_owned(),
                ".atom__button".to_owned(), "input".to_owned(),
                ".meta__extras".to_owned(), ".content__headline-showcase-wrapper".to_owned(),
                ".fc-container__header".to_owned(),
                "figure.element-embed".to_owned(),
                ".vjs-control-text".to_owned(),
            ],
            delazify: true,
            fix_embeds: true,
            content_script: Some(r#"<script>
            [...document.querySelectorAll("[data-src-background]")]
                .map(d => d.src = d.dataset["src-background"].replace("background-image: url", "").replace(/[\(\)]/g, ""))
            </script>"#.to_owned()),
            preprocess: vec![
                AttributeRewrite {
                    selector: ".vjs-big-play-button[style]".to_owned(),
                    attribute: Some(("style".to_owned(), "data-src-background".to_owned())),
                    element_name: "img".to_owned()
                }
            ],
        })});

        self.add_configuration(SpeedReaderConfig {
            domain: "washingtonpost.com".to_owned(),
            url_rules: vec![
                r#"/washingtonpost.com\/.*\/(\d){4}\/(\d){2}\/(\d){2}\/\w+/"#.to_owned(),
                r#"||washingtonpost.com*_story.html"#.to_owned(),
                r#"! travel pages currently handled poorly"#.to_owned(),
                r#"@@||washingtonpost.com/travel"#.to_owned(),
                // r#"||thelily.com/*/"#.to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    "main > header".to_owned(),
                    "main > article .byline".to_owned(),
                    "main > article [data-qa=\"timestamp\"]".to_owned(),
                    "main > article figure".to_owned(),
                    ".article-body".to_owned(),
                    ".ent-article-body".to_owned(),
                    "[data-feature-name^=\"etv3\"]".to_owned(),
                ],
                main_content_cleanup: vec![
                    "header > nav".to_owned(),
                    ".tooltip".to_owned(),
                    "[data-qa=\"article-body-ad\"]".to_owned(),
                ],
                preprocess: vec![AttributeRewrite {
                    selector: "[data-fallback-image-url]".to_owned(),
                    attribute: Some(("data-fallback-image-url".to_owned(), "src".to_owned())),
                    element_name: "img".to_owned(),
                }],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "foxnews.com".to_owned(),
            url_rules: vec![
                r#"@@||video.foxnews.com"#.to_owned(),
                // r#"||foxbusiness.com/business-leaders/*"#.to_owned(),
                // r#"||foxbusiness.com/lifestyle/*"#.to_owned(),
                // r#"||foxbusiness.com/markets/*"#.to_owned(),
                // r#"||foxbusiness.com/money/*"#.to_owned(),
                // r#"||foxbusiness.com/politics/*"#.to_owned(),
                // r#"||foxbusiness.com/sports/*"#.to_owned(),
                // r#"||foxbusiness.com/technology/*"#.to_owned(),
                r#"||foxnews.com/auto/*"#.to_owned(),
                r#"||foxnews.com/entertainment/*"#.to_owned(),
                r#"||foxnews.com/faith-values/*"#.to_owned(),
                r#"||foxnews.com/food-drink/*"#.to_owned(),
                r#"||foxnews.com/great-outdoors/*"#.to_owned(),
                r#"||foxnews.com/health/*"#.to_owned(),
                r#"||foxnews.com/lifestyle/*"#.to_owned(),
                r#"||foxnews.com/media/*"#.to_owned(),
                r#"||foxnews.com/opinion/*"#.to_owned(),
                r#"||foxnews.com/politics/*"#.to_owned(),
                r#"||foxnews.com/real-estate/*"#.to_owned(),
                r#"||foxnews.com/science/*"#.to_owned(),
                r#"||foxnews.com/sports/*"#.to_owned(),
                r#"||foxnews.com/tech/*"#.to_owned(),
                r#"||foxnews.com/travel/*"#.to_owned(),
                r#"||foxnews.com/us/*"#.to_owned(),
                r#"||foxnews.com/world/*"#.to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article".to_owned()],
                main_content_cleanup: vec![
                    ".sidebar".to_owned(),
                    ".article-social".to_owned(),
                    ".author-headshot".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "forbes.com".to_owned(),
            url_rules: vec![
                r#"/forbes.com\/sites\/\w+\/(\d){4}\/(\d){2}\/(\d){2}\/\w+/"#.to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article > main".to_owned(), ".body-container".to_owned()],
                main_content_cleanup: vec![
                    ".article-footer".to_owned(),
                    ".disqus-module".to_owned(),
                    ".article-sharing".to_owned(),
                    "sharing".to_owned(),
                    ".fs-author-avatar".to_owned(),
                    ".fs-icon".to_owned(),
                    ".contrib-bio button".to_owned(),
                    ".contrib-bio .contributor-about__initial-description".to_owned(),
                    "fbs-ad".to_owned(),
                    "#speechkit-io-iframe".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "cnbc.com".to_owned(),
            url_rules: vec![
                r#"/cnbc.com\/(\d){4}\/(\d){2}\/(\d){2}\/.*.html/"#.to_owned(),
                r#"||cnbc.com/select/*/"#.to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
            main_content: vec![
                "#main-article-header".to_owned(),
                "[data-module=\"ArticleBody\"]".to_owned(),
            ],
            main_content_cleanup: vec![
                ".InlineVideo-videoEmbed".to_owned()
            ],
            delazify: false,
            fix_embeds: false,
            content_script: Some(r#"<script>
              [...document.querySelectorAll("figure")].map(f => {
                let imgid = f.id.replace("ArticleBody-InlineImage-", "");
                f.querySelector("img").src = "https://image.cnbcfm.com/api/v1/image/"+imgid+"-.jpeg?w=678";
              })
            </script>"#.to_owned()),
            preprocess: vec![
                AttributeRewrite {
                    selector: "[id^=\"ArticleBody-InlineImage\"]".to_owned(),
                    attribute: None,
                    element_name: "figure".to_owned()
                },
                AttributeRewrite {
                    selector: "[id^=\"ArticleBody-InlineImage\"] .lazyload-placeholder".to_owned(),
                    attribute: None,
                    element_name: "img".to_owned()
                },
                AttributeRewrite {
                    selector: "[id^=\"ArticleBody-InlineImage\"] > div > div:not([class*=\"imagePlaceholder\"])".to_owned(),
                    attribute: None,
                    element_name: "figcaption".to_owned()
                }
            ],
        })});

        self.add_configuration(SpeedReaderConfig {
            domain: "usatoday.com".to_owned(),
            url_rules: vec![r#"||usatoday.com/story/*"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article".to_owned(), ".article-wrapper".to_owned()],
                main_content_cleanup: vec![
                    ".gnt_ss".to_owned(),
                    "svg".to_owned(),
                    "custom-style".to_owned(),
                ],
                preprocess: vec![
                    AttributeRewrite {
                        selector: "button[data-c-vpattrs]".to_owned(),
                        attribute: Some(("id".to_owned(), "id".to_owned())),
                        element_name: "div".to_owned(),
                    },
                    AttributeRewrite {
                        selector: "slide".to_owned(),
                        attribute: Some(("original".to_owned(), "src".to_owned())),
                        element_name: "img".to_owned(),
                    },
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "wsj.com".to_owned(),
            url_rules: vec![
                // r#"||www.barrons.com/articles/"#.to_owned(),
                r#"||www.wsj.com/articles/"#.to_owned(),
                // r#"||marketwatch.com/story/*"#.to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article > main".to_owned()],
                main_content_cleanup: vec![
                    "#saving-united-coupon-list".to_owned(),
                    ".author-info".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "reuters.com".to_owned(),
            url_rules: vec![r#"||reuters.com/article/*"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    ".ArticleHeader_container".to_owned(),
                    ".StandardArticleBody_body".to_owned(),
                ],
                main_content_cleanup: vec![
                    ".SmallImage_small-image".to_owned(),
                    "[class$=expand-button]".to_owned(),
                    ".Slideshow_caption".to_owned(),
                    "[role=button]".to_owned(),
                ],
                content_script: Some(
                    r#"<script>
                [...document.querySelectorAll(".LazyImage_container img")]
                    .map(i => i.src = i.src.replace(/\&w=\d+/, "&w=600"));
            </script>"#
                        .to_owned(),
                ),
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "nypost.com".to_owned(),
            url_rules: vec![r#"/nypost.com\/(\d){4}\/(\d){2}\/(\d){2}\/.*/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".article-header".to_owned(), ".slide".to_owned()],
                main_content_cleanup: vec![
                    ".no-mobile".to_owned(),
                    ".author-contact".to_owned(),
                    ".sharedaddy".to_owned(),
                    ".author-flyout".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "chron.com".to_owned(),
            url_rules: vec!["||chron.com/*/article/".to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".article-title".to_owned(), ".article-body".to_owned()],
                main_content_cleanup: vec![
                    ".hidden".to_owned(),
                    ".control-panel".to_owned(),
                    ".article-body > script".to_owned(),
                    ".caption-truncated".to_owned(),
                ],
                preprocess: vec![AttributeRewrite {
                    selector: "li.hst-resgalleryitem".to_owned(),
                    attribute: None,
                    element_name: "div".to_owned(),
                }],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "nbcnews.com".to_owned(),
            url_rules: vec![
                "||nbcnews.com/*-n*".to_owned(),
                "@@||nbcnews.com/video".to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    ".article header".to_owned(),
                    ".article article".to_owned(),
                    ".article figure".to_owned(),
                ],
                main_content_cleanup: vec![
                    ".article article svg".to_owned(),
                    "[data-test=newsletter-signup]".to_owned(),
                    "#emailSignup".to_owned(),
                    ".ad-container".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "dw.com".to_owned(),
            url_rules: vec!["||dw.com/*/a-*".to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["#bodyContent".to_owned()],
                main_content_cleanup: vec![
                    "[class$=Teaser]".to_owned(),
                    ".video".to_owned(),
                    ".relatedContent".to_owned(),
                    ".smallList".to_owned(),
                    "#sharing-bar".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "time.com".to_owned(),
            url_rules: vec![r#"/time.com\/(\d){6,}\/.*/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["main.article".to_owned()],
                main_content_cleanup: vec![
                    ".edit-link".to_owned(),
                    ".most-popular-feed".to_owned(),
                    ".inline-recirc".to_owned(),
                    ".newsletter-callout".to_owned(),
                    ".article-bottom".to_owned(),
                    ".article-small-sidebar".to_owned(),
                    ".ad".to_owned(),
                    ".component.video video:not([poster])".to_owned(),
                ],
                preprocess: vec![AttributeRewrite {
                    selector: "noscript".to_owned(),
                    attribute: None,
                    element_name: "div".to_owned(),
                }],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "cbsnews.com".to_owned(),
            url_rules: vec![
                "||cbsnews.com/news/*".to_owned(),
                "@@||cbsnews.com/live".to_owned(),
                "@@||cbsnews.com/video".to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article.content".to_owned(), "article.article".to_owned()],
                main_content_cleanup: vec![
                    ".sharebar".to_owned(),
                    ".content__cta".to_owned(),
                    "figure .embed__content--draggable".to_owned(),
                    "figure svg".to_owned(),
                    "script".to_owned(),
                    "[data-component=socialLinks]".to_owned(),
                    "[data-component=sharebar]".to_owned(),
                ],
                preprocess: vec![AttributeRewrite {
                    selector: "link[as=\"image\"]".to_owned(),
                    attribute: Some(("href".to_owned(), "src".to_owned())),
                    element_name: "img".to_owned(),
                }],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "thedailybeast.com".to_owned(),
            url_rules: vec![
              "@@||thedailybeast.com/category/".to_owned(),
              r#"/thedailybeast\.com\/(\w+-)+/"#.to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
            main_content: vec!["article.Story".to_owned(), "body > div > script:not([src]):not([type])".to_owned()],
            main_content_cleanup: vec![
                ".StandardHeader__share-buttons".to_owned(),
                ".StoryFooter".to_owned(),
                ".PullQuote__logo-icon".to_owned(),
                ".PullQuote__top-line".to_owned(),
                ".PullQuote__big-quote".to_owned(),
                "figure svg".to_owned(),
                ".SimpleAd".to_owned(),
                ".Byline__photo-link".to_owned(),
            ],
            delazify: true,
            fix_embeds: false,
            content_script: Some(r#"<script>
            [...document.querySelectorAll(".Body .LazyLoad")].map((div, i) => {
                let lazyLoad = window.__INITIAL_STATE__.body.cards.filter(c => c[0] === "pt-image" || c[0] === "pt-video-card")[i];
                if (!lazyLoad || lazyLoad[0] !== "pt-image") return;
                let figure = document.createElement("figure");
                figure.innerHTML = '<img src="https://img.thedailybeast.com/image/upload/c_crop/dpr_1.0/c_limit,w_600/fl_lossy,q_auto/' 
                    + lazyLoad[1].public_id + '"><figcaption>' 
                    + lazyLoad[1].title + ' Credit: ' 
                    + lazyLoad[1].credit + '</figcaption>';
                div.appendChild(figure);
            })
            </script>"#.to_owned()),
            preprocess: vec![
                AttributeRewrite {
                    selector: ".PullQuote".to_owned(),
                    attribute: None,
                    element_name: "blockquote".to_owned()
                }
            ],
        })});

        self.add_configuration(SpeedReaderConfig {
            domain: "businessinsider.com".to_owned(),
            url_rules: vec![r#"/businessinsider\.com\/(\w+-)+(\d){4}-(\d)/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    ".post-headline:nth".to_owned(),
                    ".byline-wrapper".to_owned(),
                    "#l-content".to_owned(),
                    ".container figure".to_owned(),
                ],
                main_content_cleanup: vec![
                    ".share-wrapper".to_owned(),
                    ".ad".to_owned(),
                    ".category-tagline".to_owned(),
                    ".popular-video".to_owned(),
                    "figure .lazy-image".to_owned(),
                    "figure .lazy-blur".to_owned(),
                ],
                preprocess: vec![AttributeRewrite {
                    selector: "figure noscript".to_owned(),
                    attribute: None,
                    element_name: "div".to_owned(),
                }],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "thehill.com".to_owned(),
            url_rules: vec![r#"/thehill\.com\/[\w-]+\/[\w-]+\/(\d){3}-.*/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    ".content-wrapper.title".to_owned(),
                    ".title-wrapper .title".to_owned(),
                    "article".to_owned(),
                ],
                main_content_cleanup: vec![
                    ".dfp-tag-wrapper".to_owned(),
                    ".rollover-block".to_owned(),
                    "#jwplayer-unmute-button".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "theatlantic.com".to_owned(),
            url_rules: vec![r#"/theatlantic.com\/.*\/(\d){4}\/(\d){2}\/.*\/\d{4,}/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article".to_owned()],
                main_content_cleanup: vec![
                    ".c-share-social".to_owned(),
                    "header .c-article-author__image".to_owned(),
                    ".c-article-writer__social-link-icon".to_owned(),
                    ".ad-boxinjector-wrapper".to_owned(),
                    ".ad".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "livemint.com".to_owned(),
            url_rules: vec![r#"/livemint.com\/.*-\d{4,}\.html/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article".to_owned(), ".contentSec".to_owned()],
                main_content_cleanup: vec![
                    ".socialHolder".to_owned(),
                    ".adHolderStory".to_owned(),
                    "a.btnClose".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "sfgate.com".to_owned(),
            url_rules: vec!["||sfgate.com/*/article/*".to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    ".article-content .article-title".to_owned(),
                    ".article-content .article-body".to_owned(),
                ],
                main_content_cleanup: vec![
                    ".asset_gallery .control-panel".to_owned(),
                    ".asset_media".to_owned(),
                    ".caption-truncated".to_owned(),
                ],
                preprocess: vec![
                    AttributeRewrite {
                        selector: ".hst-resgalleryitem".to_owned(),
                        attribute: None,
                        element_name: "figure".to_owned(),
                    },
                    AttributeRewrite {
                        selector: ".caption".to_owned(),
                        attribute: None,
                        element_name: "figcaption".to_owned(),
                    },
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "alarabiya.net".to_owned(),
            url_rules: vec![r#"/alarabiya\.net\/(\d){4}\/(\d){2}\/(\d){2}\/.*/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article".to_owned()],
                main_content_cleanup: vec!["article > img".to_owned(), ".teaser-tools".to_owned()],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "euronews.com".to_owned(),
            url_rules: vec![r#"/euronews\.com\/(\d){4}\/(\d){2}\/(\d){2}\/.*/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article".to_owned()],
                main_content_cleanup: vec![
                    "[class$=spotim]".to_owned(),
                    "article [class^=js-]".to_owned(),
                    ".teaser-tools".to_owned(),
                    "[class*=social]".to_owned(),
                    ".media__body__cartouche__mask".to_owned(),
                    ".c-font-size-switcher".to_owned(),
                    "footer".to_owned(),
                    ".c-article-meta__content-img".to_owned(),
                    ".ads".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "nationalgeographic.com".to_owned(),
            url_rules: vec![r#"/nationalgeographic\.com\/.*\/(\d){4}\/(\d){2}\/.*/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article".to_owned()],
                main_content_cleanup: vec![
                    "#smart-body__read-more".to_owned(),
                    ".lead-container__social-wrap".to_owned(),
                    ".media__caption--mobile-expanded".to_owned(),
                    ".UniversalVideo".to_owned(),
                    ".enlarge-button".to_owned(),
                ],
                preprocess: vec![AttributeRewrite {
                    selector: ".placeholder-image-wrap .picturefill".to_owned(),
                    attribute: None,
                    element_name: "img".to_owned(),
                }],
                content_script: Some(
                    r#"<script>
                [...document.querySelectorAll(".picturefill")]
                .map((e, i) => {
                e.src = JSON.parse(e.parentElement.querySelector("script").innerHTML).src;
                })
                </script>"#
                        .to_owned(),
                ),
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "latimes.com".to_owned(),
            url_rules: vec![r#"||latimes.com/*/story/*"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    ".ArticlePage-content".to_owned(),
                    ".LongFormPage-content".to_owned(),
                ],
                main_content_cleanup: vec![
                    ".NewsletterModule".to_owned(),
                    "[class*=Page-actions]".to_owned(),
                    "[class*=Page-contentFooter]".to_owned(),
                    "[class*=Page-aside]".to_owned(),
                    "[class*=Page-comments]".to_owned(),
                    ".RevContent".to_owned(),
                    ".SocialBar".to_owned(),
                    ".Enhancement > .Infobox".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "newsweek.com".to_owned(),
            url_rules: vec![r#"/newsweek\.com\/.*(\d){6,}/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    "article".to_owned(),
                    ".article-header".to_owned(),
                    "figure".to_owned(),
                ],
                main_content_cleanup: vec![
                    ".social-share".to_owned(),
                    "article .block-nw-magazine".to_owned(),
                    ".hidden-print".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "variety.com".to_owned(),
            url_rules: vec![r#"/variety\.com\/.*-(\d){6,}/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".post".to_owned()],
                main_content_cleanup: vec![
                    ".c-author__extra".to_owned(),
                    "[data-trigger=share-links-manager]".to_owned(),
                    ".pmc-contextual-player".to_owned(),
                    "footer".to_owned(),
                    ".c-ad, .admz".to_owned(),
                    "[id^=comments]".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "hollywoodreporter.com".to_owned(),
            url_rules: vec![r#"/hollywoodreporter\.com\/.*-(\d){6,}/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article".to_owned(), ".blog-post".to_owned()],
                main_content_cleanup: vec![
                    ".social-share".to_owned(),
                    ".dfp-ad".to_owned(),
                    ".blog-post aside".to_owned(),
                    ".blog-post-sections".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "bbc.com".to_owned(),
            url_rules: vec![
                r#"/bbc\.com\/.*-(\d){6,}/"#.to_owned(),
                r#"/bbc\.com\/.*\/.*\/(\d){6,}/"#.to_owned(),
                "||bbc.com/*/articles/*".to_owned(),
                "@@||bbc.com/*/live/*".to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    ".story-headline, .story-info, .story-body".to_owned(),
                    "#story-page".to_owned(),     // different format
                    "section.article".to_owned(), //another different format
                    ".programmes-page.article--individual".to_owned(), // and another one
                    "article.blocks-article".to_owned(),
                ],
                main_content_cleanup: vec![
                    "[class*=share-tools]".to_owned(),
                    ".idt2".to_owned(),
                    ".off-screen".to_owned(),
                    "#share-tools-top".to_owned(),
                    ".story-share".to_owned(),
                    ".article__footer".to_owned(),
                    "article header img".to_owned(),
                    ".drop-capped".to_owned(),
                ],
                preprocess: vec![
                    AttributeRewrite {
                        selector: ".js-delayed-image-load".to_owned(),
                        attribute: None,
                        element_name: "img".to_owned(),
                    },
                    AttributeRewrite {
                        selector: "a.replace-image".to_owned(),
                        attribute: Some(("href".to_owned(), "src".to_owned())),
                        element_name: "img".to_owned(),
                    },
                    AttributeRewrite {
                        selector: ".blocks-image".to_owned(),
                        attribute: None,
                        element_name: "img".to_owned(),
                    },
                ],
                fix_embeds: true,
                content_script: Some(
                    r#"<script>
                [...document.querySelectorAll("img[data-src].blocks-image")].map((e, i) => {
                    if (e.dataset["src"]) {
                        e.src = e.dataset["src"].replace("{width}", "800xn");
                    }
                });
                [...document.querySelectorAll("img[data-src].sp-lazyload")].map((e, i) => {
                    if (e.dataset["src"]) {
                        e.src = e.dataset["src"].replace("{width}", "800").replace("{hidpi}","");
                    }
                })
                </script>"#
                        .to_owned(),
                ),
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "bbc.co.uk".to_owned(),
            url_rules: vec![
                r#"/bbc\.co.uk\/.*-(\d){6,}/"#.to_owned(),
                r#"/bbc\.co.uk\/.*\/.*\/(\d){6,}/"#.to_owned(),
                "||bbc.co.uk/*/articles/*".to_owned(),
                "@@||bbc.co.uk/*/live/*".to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    ".story-headline, .story-info, .story-body".to_owned(),
                    "#story-page".to_owned(),     // different format
                    "section.article".to_owned(), //another different format
                    ".programmes-page.article--individual".to_owned(), // and another one
                    "article.blocks-article".to_owned(),
                ],
                main_content_cleanup: vec![
                    "[class*=share-tools]".to_owned(),
                    ".idt2".to_owned(),
                    ".off-screen".to_owned(),
                    "#share-tools-top".to_owned(),
                    ".story-share".to_owned(),
                    ".article__footer".to_owned(),
                    "article header img".to_owned(),
                    ".drop-capped".to_owned(),
                ],
                preprocess: vec![
                    AttributeRewrite {
                        selector: ".js-delayed-image-load".to_owned(),
                        attribute: None,
                        element_name: "img".to_owned(),
                    },
                    AttributeRewrite {
                        selector: "a.replace-image".to_owned(),
                        attribute: Some(("href".to_owned(), "src".to_owned())),
                        element_name: "img".to_owned(),
                    },
                    AttributeRewrite {
                        selector: ".blocks-image".to_owned(),
                        // attribute: Some(("src".to_owned(), "src".to_owned())),
                        attribute: None,
                        element_name: "img".to_owned(),
                    },
                ],
                fix_embeds: true,
                content_script: Some(
                    r#"<script>
                [...document.querySelectorAll("img[data-src].blocks-image")].map((e, i) => {
                    if (e.dataset["src"]) {
                        e.src = e.dataset["src"].replace("{width}", "800xn");
                    }
                });
                [...document.querySelectorAll("img[data-src].sp-lazyload")].map((e, i) => {
                    if (e.dataset["src"]) {
                        e.src = e.dataset["src"].replace("{width}", "800").replace("{hidpi}","");
                    }
                })
                </script>"#
                        .to_owned(),
                ),
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "reddit.com".to_owned(),
            url_rules: vec!["||reddit.com/r/*/comments/*".to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".Post".to_owned(), ".Comment".to_owned()],
                main_content_cleanup: vec![
                    ".Post button".to_owned(),
                    ".Comment button".to_owned(),
                    ".Comment svg".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "weather.com".to_owned(),
            url_rules: vec!["||weather.com/*/news/*".to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".article-wrapper".to_owned()],
                main_content_cleanup: vec![],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "huffingtonpost.co.uk".to_owned(),
            url_rules: vec!["||huffingtonpost.co.uk/entry/*".to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".entry__content".to_owned(), ".entry__header".to_owned()],
                main_content_cleanup: vec![
                    ".share-bar".to_owned(),
                    ".ad_spot".to_owned(),
                    ".advertisement-label".to_owned(),
                    ".entry__content script".to_owned(),
                    ".top-media iframe".to_owned(),
                    "aside.rail".to_owned(),
                    ".cli-related-articles".to_owned(),
                    ".js-react-hydrator".to_owned(),
                ],
                preprocess: vec![
                    AttributeRewrite {
                        selector: ".vdb_player[data-placeholder]".to_owned(),
                        attribute: Some(("data-placeholder".to_owned(), "src".to_owned())),
                        element_name: "img".to_owned(),
                    },
                    AttributeRewrite {
                        selector: ".embed-asset div[style]".to_owned(),
                        attribute: Some(("style".to_owned(), "data-style".to_owned())),
                        element_name: "div".to_owned(),
                    },
                    AttributeRewrite {
                        selector: ".embed-asset iframe[style]".to_owned(),
                        attribute: Some(("style".to_owned(), "data-style".to_owned())),
                        element_name: "iframe".to_owned(),
                    },
                ],
                content_script: Some(
                    r#"<script>
                [...document.querySelectorAll(".embed-asset [data-style]")]
                    .map(e => e.style = e.dataset["style"]);
                </script>"#
                        .to_owned(),
                ),
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "huffpost.com".to_owned(),
            url_rules: vec!["||huffpost.com/entry/*".to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".entry__content".to_owned(), ".entry__header".to_owned()],
                main_content_cleanup: vec![
                    ".share-bar".to_owned(),
                    ".ad_spot".to_owned(),
                    ".advertisement-label".to_owned(),
                    ".entry__content script".to_owned(),
                    ".top-media iframe".to_owned(),
                    "aside.rail".to_owned(),
                    ".cli-related-articles".to_owned(),
                    ".js-react-hydrator".to_owned(),
                ],
                preprocess: vec![
                    AttributeRewrite {
                        selector: ".vdb_player[data-placeholder]".to_owned(),
                        attribute: Some(("data-placeholder".to_owned(), "src".to_owned())),
                        element_name: "img".to_owned(),
                    },
                    AttributeRewrite {
                        selector: ".embed-asset div[style]".to_owned(),
                        attribute: Some(("style".to_owned(), "data-style".to_owned())),
                        element_name: "div".to_owned(),
                    },
                    AttributeRewrite {
                        selector: ".embed-asset iframe[style]".to_owned(),
                        attribute: Some(("style".to_owned(), "data-style".to_owned())),
                        element_name: "iframe".to_owned(),
                    },
                ],
                content_script: Some(
                    r#"<script>
                [...document.querySelectorAll(".embed-asset [data-style]")]
                    .map(e => e.style = e.dataset["style"]);
                </script>"#
                        .to_owned(),
                ),
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "bloomberg.com".to_owned(),
            url_rules: vec![
                "||bloomberg.com/*/articles/*".to_owned(),
                "||bloomberg.com/*/features/*".to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".article-content".to_owned(), ".feature-article".to_owned()],
                main_content_cleanup: vec![
                    ".right-rail".to_owned(),
                    ".article-newsfeed".to_owned(),
                    ".gateway-mobile-lede-text".to_owned(),
                    ".video-player__overlay".to_owned(),
                    ".left-column".to_owned(),
                    ".share-article-button".to_owned(),
                    ".text-to-speech".to_owned(),
                    ".first-paragraph-image".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "usnews.com".to_owned(),
            url_rules: vec!["||usnews.com/*/articles/*".to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".content".to_owned(), ".feature-article".to_owned()],
                main_content_cleanup: vec![
                    ".right-rail".to_owned(),
                    ".article-newsfeed".to_owned(),
                    ".flex".to_owned(),
                    ".sticky-heading".to_owned(),
                    "svg[class^=Credit]".to_owned(),
                    "button".to_owned(),
                    "[class*=-hide]".to_owned(),
                    "[class*=Hide-]".to_owned(),
                    "[class*=LoadMore]".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "smh.com.au".to_owned(),
            url_rules: vec![r#"/smh\.com\.au\/.*\/(\w+-){3,}-(\d{6})-(p\d{3,})\.html/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article".to_owned()],
                main_content_cleanup: vec![".noPrint".to_owned()],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "chicagotribune.com".to_owned(),
            url_rules: vec!["@@||chicagotribune.com/*-story.html".to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec!["article".to_owned()],
                main_content_cleanup: vec![
                    ".sharebar".to_owned(),
                    "[data-type=recommender]".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "aljazeera.com".to_owned(),
            url_rules: vec![
                "@@||aljazeera.com/news/*.html".to_owned(),
                "@@||aljazeera.com/indepth/*.html".to_owned(),
            ],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    ".article-heading".to_owned(),
                    ".main-article-body".to_owned(),
                    ".article-gallery-sec".to_owned(),
                ],
                main_content_cleanup: vec![
                    ".article-embedded-card".to_owned(),
                    ".article-readToMe-share-block".to_owned(),
                    ".article-heading-author-name img".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "dailycaller.com".to_owned(),
            url_rules: vec![r#"/dailycaller\.com\/(\d){4}\/(\d){2}\/(\d){2}\/.*/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![
                    "article header".to_owned(),
                    "article .article-content".to_owned(),
                ],
                main_content_cleanup: vec!["header button".to_owned(), "footer".to_owned()],
                ..RewriteRules::default()
            }),
        });

        self.add_configuration(SpeedReaderConfig {
            domain: "theonion.com".to_owned(),
            url_rules: vec![r#"/theonion\.com\/.*-(\d){6,}/"#.to_owned()],
            declarative_rewrite: Some(RewriteRules {
                main_content: vec![".js_post-content".to_owned(), "div header".to_owned()],
                main_content_cleanup: vec![
                    ".js_share-tools".to_owned(),
                    ".post-tools-wrapper".to_owned(),
                    ".js_tag-dropdown".to_owned(),
                    ".magnifier".to_owned(),
                ],
                ..RewriteRules::default()
            }),
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn default_whitelist_no_config() {
        let whitelist = Whitelist::default();
        assert!(whitelist.map.is_empty());
        let config = whitelist.get_configuration("example.com");
        assert!(config.is_none());
    }

    #[test]
    pub fn get_some_configuration() {
        let mut whitelist = Whitelist::default();
        whitelist.add_configuration(SpeedReaderConfig {
            domain: "example.com".to_owned(),
            url_rules: vec![
                r#"||example.com/article"#.to_owned(),
                r#"@@||example.com/article/video"#.to_owned(),
            ],
            declarative_rewrite: None,
        });
        let config = whitelist.get_configuration("example.com");
        assert!(config.is_some());
    }

    #[test]
    pub fn get_some_subdomain_configuration() {
        let mut whitelist = Whitelist::default();
        whitelist.add_configuration(SpeedReaderConfig {
            domain: "example.com".to_owned(),
            url_rules: vec![
                r#"||example.com/article"#.to_owned(),
                r#"@@||example.com/article/video"#.to_owned(),
            ],
            declarative_rewrite: None,
        });
        let config = whitelist.get_configuration("www.example.com");
        assert!(config.is_some());
    }

    #[test]
    pub fn url_rules_collected() {
        let mut whitelist = Whitelist::default();
        whitelist.add_configuration(SpeedReaderConfig {
            domain: "example.com".to_owned(),
            url_rules: vec![
                r#"||example.com/article"#.to_owned(),
                r#"@@||example.com/article/video"#.to_owned(),
            ],
            declarative_rewrite: None,
        });
        whitelist.add_configuration(SpeedReaderConfig {
            domain: "example.net".to_owned(),
            url_rules: vec![r#"||example.net/article"#.to_owned()],
            declarative_rewrite: None,
        });
        let rules = whitelist.get_url_rules();
        assert_eq!(rules.len(), 3);
    }

    #[test]
    pub fn conflicting_insert_overrides() {
        let mut whitelist = Whitelist::default();
        whitelist.add_configuration(SpeedReaderConfig {
            domain: "example.com".to_owned(),
            url_rules: vec![
                r#"||example.com/article"#.to_owned(),
                r#"@@||example.com/article/video"#.to_owned(),
            ],
            declarative_rewrite: None,
        });
        whitelist.add_configuration(SpeedReaderConfig {
            domain: "example.com".to_owned(),
            url_rules: vec![r#"||example.com/news"#.to_owned()],
            declarative_rewrite: None,
        });
        assert_eq!(whitelist.map.len(), 1);
        let config = whitelist.get_configuration("example.com");
        assert!(config.is_some());
        assert_eq!(
            config.unwrap().url_rules,
            vec!["||example.com/news".to_owned()]
        );
    }
}