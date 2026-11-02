<div dir="rtl" align="right">

# شروع کار

سه راه برای استفاده هست: سیمرغ روی README پروفایل، خط فرمان، و کتابخانه. هر کدام
مستقل از دیگری کار می‌کند. (English guide: [getting-started.md](getting-started.md))

## روی README پروفایل

گیت‌هاب README مخزنی را که به نام کاربری شما نام‌گذاری شده، مثلاً `octocat/octocat`،
در صفحهٔ پروفایل نشان می‌دهد. برای اینکه سیمرغی داشته باشید که هر شب دوباره کشیده شود:

1. فایل [examples/profile-workflow.yml](../examples/profile-workflow.yml) را به
   همان مخزن، با نام `.github/workflows/simurgh.yml`، کپی کنید.
2. سیمرغ را در `README.md` همان مخزن بگذارید:

   ```html
   <img src="simurgh/simurgh.gif" width="256" alt="سیمرغ من" />
   ```

   GIF دقیقاً دو برابر اندازهٔ پیکسل‌های سیمرغ کشیده می‌شود، پس `width="256"` آن را
   در اندازهٔ اصلی و بدون محو شدن نشان می‌دهد.

3. ورک‌فلو را یک بار از تب Actions مخزن اجرا کنید تا اولین سیمرغ بدون انتظار برای
   نیمه‌شب ظاهر شود.

ورک‌فلو کمی بعد از نیمه‌شب UTC اجرا می‌شود، پس هر سیمرغ روزی را نشان می‌دهد که
همین الان تمام شده، و هر بار هم که فایل ورک‌فلو عوض شود اجرا می‌شود. مرحلهٔ
`simurgh` آن همین Action این مخزن است: فعالیت عمومی شما را با توکن داخلی ورک‌فلو
جمع می‌کند و `simurgh.gif`، `simurgh.png` و `preview.html` را در `simurgh/` می‌سازد.
خود Action هرگز کامیت نمی‌کند. آخرین مرحلهٔ ورک‌فلو فقط وقتی `simurgh/simurgh.gif`
تغییر کرده باشد آن را کامیت می‌کند، پس سیمرغ بدون تغییر کامیتی اضافه نمی‌کند.

| ورودی | پیش‌فرض | معنی |
|---|---|---|
| `login` | مالک مخزن | حسابی که سیمرغش کشیده می‌شود |
| `body` | `zarrin` | یکی از `zarrin`، `firuzeh`، `lajvard`، `sepid`، `shab` یا `arghavan` |
| `output-dir` | `simurgh` | پوشه‌ای که `simurgh.gif`، `simurgh.png` و `preview.html` در آن ساخته می‌شود |
| `token` | توکن داخلی | توکن برای خواندن داده‌های عمومی حساب |

برای عوض کردن رنگ‌بندی، `body` را در نسخهٔ خودتان از ورک‌فلو ویرایش کنید؛ با
push کردن تغییر، ورک‌فلو اجرا می‌شود و سیمرغ دوباره کشیده می‌شود. اگر `output-dir`
را عوض کردید، مسیر مرحلهٔ کامیت و تصویر README را هم هماهنگ کنید. مثال از
`OWNER/REPO@v0.1.0` استفاده می‌کند: به‌جای `OWNER/REPO` نام مخزنی را بگذارید که
نسخهٔ شما از این پروژه در آن است. Action مخزن خودش را خودش پیدا می‌کند و
باینری انتشار هم‌نسخهٔ خودش را دانلود می‌کند، پس آن را روی یک تگ انتشار همان
مخزن ارجاع دهید.

گیت‌هاب ورک‌فلوهای زمان‌بندی‌شدهٔ مخزن عمومی را بعد از ۶۰ روز بی‌فعالیتی (مثلاً
بدون push) متوقف می‌کند. سیمرغی که در یک دورهٔ طولانی ساکت تغییر نمی‌کند کامیتی
نمی‌سازد، پس زمان‌بندی‌اش می‌تواند متوقف شود و خودبه‌خود ادامه نمی‌یابد. آن را از
تب Actions یا با `gh workflow enable simurgh.yml` دوباره فعال کنید.

## از خط فرمان

به یکی از دو روش نصب کنید. اگر Rust با نسخهٔ `rust-version` در
[Cargo.toml](../Cargo.toml) یا بالاتر دارید، از داخل پوشهٔ مخزن:

```sh
cargo install --path . --locked
```

(اگر کریت را روی crates.io منتشر کنید، `cargo install <نام-کریت> --locked` هم کار
می‌کند.) یا از صفحهٔ Releases نسخهٔ خودتان از مخزن، آرشیو مناسب سیستم‌تان را
دانلود کنید: `.tar.gz` برای لینوکس و macOS یا `.zip` برای ویندوز. کنار هر آرشیو
یک فایل `.sha256` هست؛ دانلود را با `shasum -a 256 -c` در macOS،
`sha256sum -c` در لینوکس یا `certutil -hashfile` در ویندوز بررسی کنید و بعد
`simurgh` را در `PATH` بگذارید. macOS ممکن است از باز کردن باینری دانلودشده
خودداری کند؛ دستور `xattr -d com.apple.quarantine simurgh` این پرچم را پاک می‌کند.

جمع‌آوری داده از طریق [GitHub CLI](https://cli.github.com) انجام می‌شود، پس آن را
نصب کنید و یک بار با `gh auth login` وارد شوید. سپس:

```sh
simurgh collect octocat --output snapshot.json
simurgh render snapshot.json --output-dir simurgh --body lajvard
```

فایل `simurgh/preview.html` را در مرورگر باز کنید تا GIF و تصویر ثابت را روی
رنگ‌های روشن و تیرهٔ صفحهٔ گیت‌هاب ببینید. گزینهٔ `--body` یکی از `zarrin`
(پیش‌فرض)، `firuzeh`، `lajvard`، `sepid`، `shab` یا `arghavan` را می‌گیرد.

`collect` فقط داده‌های عمومی را می‌خواند: مخازن عمومی حساب، مخازن عمومی که در آن‌ها
مشارکت کرده، رنگ گیت‌هاب برای هر زبان، و مشارکت‌های روزانهٔ حساب از زمان ساخته
شدنش تا آخرین روز کامل UTC. snapshot را در خروجی استاندارد می‌نویسد، یا با
`--output` فایل را فقط بعد از موفقیت جمع‌آوری جایگزین می‌کند. در GitHub Actions
کافی است `GH_TOKEN` را روی توکن ورک‌فلو بگذارید. `render` ابتدا همه‌چیز را در حافظه
می‌کشد و هر فایل را از طریق یک فایل موقت جایگزین می‌کند، پس اجرای ناموفق فایل‌های
قبلی را سالم نگه می‌دارد. هر دو خطاها را در خروجی خطای استاندارد چاپ می‌کنند و با
کد ۱ خارج می‌شوند؛ استفادهٔ نادرست با کد ۲. `simurgh --help` راهنمای کامل و
`simurgh --version` نسخه را چاپ می‌کند.

## به‌عنوان کتابخانه

همین کریت یک کتابخانه هم هست. مستنداتش را با `cargo doc --open` بخوانید، یا اگر
منتشرش کردید در docs.rs. آن را کنار `serde_json` که snapshot را می‌خواند اضافه
کنید، از یک مسیر یا از crates.io:

```sh
cargo add simurgh --path /path/to/your/copy
cargo add serde_json
```

خط لوله یک‌طرفه است: یک `AccountSnapshot` بخوانید یا جمع کنید، رشد سیمرغ را
محاسبه کنید، رفتار روز را انتخاب کنید، فریم‌ها را بکشید و کدگذاری کنید:

```rust
use simurgh::behavior::select_behavior;
use simurgh::coat::SimurghBody;
use simurgh::history::AccountSnapshot;
use simurgh::development::derive_simurgh_development;
use simurgh::motion::frame_count;
use simurgh::output::{encode_gif, encode_png};
use simurgh::render::{render_flight_frame, render_simurgh};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot_json = std::fs::read_to_string("snapshot.json")?;
    let account_snapshot: AccountSnapshot = serde_json::from_str(&snapshot_json)?;
    let simurgh_development = derive_simurgh_development(&account_snapshot)?;
    let simurgh_body = SimurghBody::Lajvard;
    let behavior = select_behavior(&simurgh_development);
    let flight_frames: Vec<_> = (0..frame_count(behavior))
        .map(|frame_index| render_flight_frame(&simurgh_development, simurgh_body, behavior, frame_index))
        .collect();
    std::fs::write("simurgh.gif", encode_gif(&flight_frames)?)?;
    std::fs::write("simurgh.png", encode_png(&render_simurgh(&simurgh_development, simurgh_body))?)?;
    Ok(())
}
```

برای جمع‌آوری داخل خود برنامه به‌جای خواندن فایل، `collect::collect_account_snapshot`
را با `collect::GhCli` و تاریخ امروزِ UTC صدا بزنید. هر چه بعد از جمع‌آوری می‌آید
قطعی است و ورودی/خروجی انجام نمی‌دهد، پس یک snapshot همیشه یک سیمرغ را می‌کشد.
تغییرهای ناسازگار API نسخهٔ اصلی (major) را بالا می‌برند.

## انتشار نسخهٔ خودتان روی گیت‌هاب

این مسیر کامل از فایل‌های روی کامپیوتر شما تا یک سیمرغ روی پروفایلتان است. به‌جای
`OWNER` نام کاربری گیت‌هاب خودتان (یا سازمان) را بنویسید.

**لازم دارید:** یک حساب گیت‌هاب، [Git](https://git-scm.com) و در صورت تمایل
[GitHub CLI](https://cli.github.com) (دستور `gh`). برای انتشار به Rust نیازی نیست.

### ۱. ساخت مخزن و push

در github.com گزینهٔ **New repository** را بزنید، نامش را `simurgh` بگذارید، آن را
**Public** کنید و گزینه‌های «Add a README»، «.gitignore» و «license» را **تیک نزنید**
(پروژه خودش دارد). سپس در پوشهٔ پروژه:

```sh
git init
git add .
git commit -m "Initial commit"
git branch -M main
git remote add origin https://github.com/OWNER/simurgh.git
git push -u origin main
```

با GitHub CLI یک دستور همین کار را می‌کند:
`gh repo create simurgh --public --source=. --push`.

کامیت‌ها ایمیلی را نشان می‌دهند که در تنظیمات Git دارید. برای اینکه آدرس واقعی‌تان
دیده نشود، از صفحهٔ Settings و Emails گیت‌هاب آدرس
`ID+USERNAME@users.noreply.github.com` را بردارید:
`git config user.email "ID+USERNAME@users.noreply.github.com"`.

### ۲. بررسی اجرای ورک‌فلوها

تب **Actions** را باز کنید. اگر گیت‌هاب خواست ورک‌فلوها را فعال کنید، فعال کنید.
ورک‌فلوهای `CI` و `End to end` باید سبز شوند. اگر اجرایی شکست خورد، آن را باز کنید و
مرحلهٔ قرمز را بخوانید؛ لاگ می‌گوید مشکل چیست.

### ۳. انتشار اولین نسخه (Release)

Action برنامهٔ خودش را از یک GitHub Release دانلود می‌کند، پس تا Release نباشد کسی
نمی‌تواند از Action استفاده کند. تگ باید با نسخهٔ `Cargo.toml` یکی باشد (`0.1.0`
می‌شود `v0.1.0`) و `CHANGELOG.md` باید بخش `## [0.1.0]` داشته باشد؛ هر دو الان
درست‌اند.

```sh
git tag v0.1.0
git push origin v0.1.0
```

تب Actions را باز کنید و صبر کنید ورک‌فلوی `Release` سبز شود. بعد صفحهٔ **Releases**
نسخهٔ `v0.1.0` را با آرشیوهای لینوکس، macOS و ویندوز نشان می‌دهد. اگر ورک‌فلو
نتوانست Release بسازد، به Settings، Actions، General بروید و زیر «Workflow
permissions» گزینهٔ **Read and write permissions** را انتخاب کنید و دوباره اجرایش
کنید.

### ۴. پیدا شدن و فورک شدن آسان

در صفحهٔ اصلی مخزن روی چرخ‌دنده کنار **About** بزنید و یک توضیح و چند topic بگذارید،
مثلاً `github-profile`، `pixel-art`، `persian`، `simurgh`، `rust`،
`github-actions`. در Settings و General تیک **Template repository** را بزنید تا هر
کسی با دکمهٔ **Use this template** یک نسخه از آن شروع کند.

### ۵. گذاشتن سیمرغ روی پروفایل خودتان

۱. مخزنی بسازید که نامش دقیقاً مثل نام کاربری شماست و یک `README.md` دارد. README
   آن در صفحهٔ پروفایلتان نشان داده می‌شود.
۲. در آن فایل `.github/workflows/simurgh.yml` را از
   [`examples/profile-workflow.yml`](../examples/profile-workflow.yml) بسازید و
   `OWNER/REPO@v0.1.0` را به `OWNER/simurgh@v0.1.0` تغییر دهید. اگر خواستید `body` را
   انتخاب کنید.
۳. در `README.md` آن اضافه کنید:

   ```html
   <img src="simurgh/simurgh.gif" width="256" alt="سیمرغ من" />
   ```

۴. تب Actions مخزن پروفایل را باز کنید، ورک‌فلوی **Simurgh** را انتخاب کنید و
   **Run workflow** را بزنید. بعد از حدود یک دقیقه `simurgh/simurgh.gif` ساخته
   می‌شود و پروفایلتان پرنده را نشان می‌دهد. از آن به بعد هر شب خودش دوباره کشیده
   می‌شود.

### ۶. انتشار نسخهٔ جدید در آینده

بعد از تغییری که کاربران باید دریافت کنند:

۱. `version` را در `Cargo.toml` بالا ببرید (مثلاً `0.1.1`).
۲. بخش `## [0.1.1]` را بالای `CHANGELOG.md` و زیر `## [Unreleased]` اضافه کنید.
۳. کامیت و push کنید، بعد `git tag v0.1.1` و `git push origin v0.1.1`.
۴. در ورک‌فلوی پروفایل `uses:` را روی `@v0.1.1` ببرید.

### ۷. اختیاری: انتشار روی crates.io

متغیر مخزن `PUBLISH_TO_CRATES_IO` را با مقدار `true` بسازید (Settings، Secrets and
variables، Actions، Variables) و انتشار مورد اعتماد (trusted publishing) را برای
کریت در crates.io تنظیم کنید. بدون آن متغیر، ورک‌فلوی انتشار هرگز به crates.io دست
نمی‌زند.

### اگر مشکلی پیش آمد

| پیام یا نشانه | علت و راه‌حل |
| --- | --- |
| `release v0.1.0 has no asset ...` | Release هنوز ساخته نشده یا در حال ساخت است. مرحلهٔ ۳ را کامل کنید و صبر کنید سبز شود. |
| `tag ... does not match Cargo.toml version` | تگ و `version` فرق دارند. نسخه را درست کنید، یا تگ را پاک کنید و دوباره تگ بزنید. |
| `CHANGELOG.md has no section for ...` | برای نسخهٔ تگ یک بخش `## [x.y.z]` اضافه کنید. |
| `Resource not accessible by integration` | به ورک‌فلو دسترسی نوشتن بدهید: Settings، Actions، General، Workflow permissions. |
| پروفایل تصویری نشان نمی‌دهد | ورک‌فلو را یک بار دستی اجرا کنید (مرحلهٔ ۵) و بررسی کنید مسیر تصویر در `README.md` با `output-dir` یکی باشد. |
| به‌روزرسانی شبانه متوقف شد | گیت‌هاب زمان‌بندی مخزن عمومی را بعد از ۶۰ روز بی‌فعالیتی متوقف می‌کند. ورک‌فلو را در تب Actions دوباره فعال کنید. |

</div>
