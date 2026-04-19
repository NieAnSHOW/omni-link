use regex::Regex;
use std::sync::LazyLock;

struct PlatformRule {
    platform: &'static str,
    patterns: &'static [ &'static LazyLock<Regex>],
}

static RULES: &[PlatformRule] = &[
    PlatformRule { platform: "youtube", patterns: &[&YOUTUBE1, &YOUTUBE2] },
    PlatformRule { platform: "bilibili", patterns: &[&BILIBILI1, &BILIBILI2] },
    PlatformRule { platform: "twitter", patterns: &[&TWITTER1, &TWITTER2] },
    PlatformRule { platform: "weibo", patterns: &[&WEIBO1, &WEIBO2] },
    PlatformRule { platform: "zhihu", patterns: &[&ZHIHU] },
    PlatformRule { platform: "juejin", patterns: &[&JUEJIN] },
    PlatformRule { platform: "github", patterns: &[&GITHUB] },
];

static YOUTUBE1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"youtube\.com").unwrap());
static YOUTUBE2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"youtu\.be").unwrap());
static BILIBILI1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"bilibili\.com").unwrap());
static BILIBILI2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"b23\.tv").unwrap());
static TWITTER1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"twitter\.com").unwrap());
static TWITTER2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"x\.com").unwrap());
static WEIBO1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"weibo\.com").unwrap());
static WEIBO2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"weibo\.cn").unwrap());
static ZHIHU: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"zhihu\.com").unwrap());
static JUEJIN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"juejin\.cn").unwrap());
static GITHUB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"github\.com").unwrap());

pub fn identify_platform(url: &str) -> &'static str {
    for rule in RULES {
        if rule.patterns.iter().any(|p| p.is_match(url)) {
            return rule.platform;
        }
    }
    "general"
}
