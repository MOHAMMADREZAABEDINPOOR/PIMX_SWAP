<div align="center">

<img src="assets/readme/hero.gif" width="1200" height="480" alt="PIMX SWAP — animated 3D keyboard with layout switching arrows" />

**[🌐 English](README.md) · [🇮🇷 فارسی](README.fa.md)**

[**⬇️ دانلود فایل نصب ویندوز**](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/PIMXSWAP-Setup.exe) · [Releases](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases)

</div>

<div dir="rtl">

# ⌨️ PIMX SWAP — یک میانبر، چیدمان درست

یک جمله نوشتی و آخرش متوجه شدی کیبورد روی زبان اشتباه بوده؟ متن را انتخاب کن و **Ctrl + Shift + Space** بزن. PIMX SWAP کلیدهای متن را به چیدمان درست برمی‌گرداند؛ روی ویندوز، به‌صورت محلی، بدون نیاز به حساب یا سرویس ابری.

**`sghl` → `سلام`** · **`اثممخ` → `hello`**

برای استفاده فقط فایل نصب را دانلود کن؛ ابزارهای برنامه‌نویسی لازم نیست.

| در یک نگاه | جزئیات |
|:---|:---|
| 🪟 پلتفرم | ویندوز ۱۰ / ۱۱ · x64 |
| 📦 نصب | [PIMXSWAP-Setup.exe](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/PIMXSWAP-Setup.exe) · 1.41 MiB |
| 🪶 رم در پس‌زمینه | حدود **۱۳ MiB** در اندازه‌گیری محلی حالت پس‌زمینه (12.85 MiB) |
| 🔒 پردازش متن | تبدیل متن و تشخیص زبان به‌صورت محلی |
| 🌐 رابط | انگلیسی و فارسی · پوسته‌ها · ویرایشگر جمع‌وجور |

[⬇️ نصب](#install) · [⚡ استفاده](#use) · [🪶 مصرف رم](#resources) · [🛠️ توسعه](#development)

<a id="install"></a>

## ⬇️ دانلود، نصب، استفاده

1. **[PIMXSWAP-Setup.exe را دانلود کن](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/PIMXSWAP-Setup.exe)** یا صفحهٔ [آخرین Release](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest) را باز کن.
2. فایل نصب را اجرا کن؛ برنامه برای کاربر فعلی ویندوز نصب می‌شود و میانبرهای برنامه را می‌سازد.
3. PIMXSWAP را باز کن؛ زبان رابط و چیدمان مبدأ و مقصد را انتخاب کن.
4. متنِ تایپ‌شده با چیدمان اشتباه را انتخاب کن و **Ctrl + Shift + Space** بزن.

پیش‌نیاز اجرای رابط، **Microsoft Edge WebView2 Runtime** است. اگر نصب نباشد، نصب‌کننده آن را از Microsoft دانلود می‌کند؛ بعد از نصب، تبدیل متن آفلاین انجام می‌شود. فایل نصب فعلی امضای دیجیتال ناشر ندارد و ویندوز ممکن است پیام تأیید ناشر نشان دهد.

| فایل | کاربرد |
|:---|:---|
| [PIMXSWAP-Setup.exe](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/PIMXSWAP-Setup.exe) | انتخاب معمول؛ نصب، میانبرهای ویندوز و حذف از Settings |
| [PIMXSWAP-Windows-x64.zip](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/PIMXSWAP-Windows-x64.zip) | نسخهٔ پرتابل؛ استخراج پوشه و اجرای PIMXSWAP.exe؛ همچنان به WebView2 نیاز دارد |
| [SHA256SUMS.txt](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases/latest/download/SHA256SUMS.txt) | checksum فایل‌ها برای بررسی صحت دانلود |

برای به‌روزرسانی، نصب‌کنندهٔ نسخهٔ جدید را اجرا کن؛ به‌روزرسانی خودکار تنظیم نشده است. تنظیمات کاربر هنگام حذف حفظ می‌شوند.

<a id="use"></a>

## ⚡ از متن اشتباه به متن درست

| میانبر پیش‌فرض | رفتار |
|:---|:---|
| **Ctrl + Shift + Space** | اصلاح خودکار متن انتخاب‌شده با شواهد محلی زبان |
| **Ctrl + Alt + Shift + Space** | تبدیل مستقیم با چیدمان مبدأ و مقصد تنظیم‌شده |
| **Ctrl + Alt + P** | باز کردن ویرایشگر |
| **Ctrl + Enter** | تبدیل متن در ویرایشگر داخلی |

همهٔ میانبرها در Settings قابل تغییرند. بدون انتخاب متن، برنامه ابتدا بخش خط قبل از نشانگر و سپس کلمهٔ قبلی را امتحان می‌کند؛ برای چند خط، متن را انتخاب کن. اگر نتیجه مبهم باشد، انتخابگر پیشنهادها باز می‌شود.

از حالت جمع‌وجور ویرایشگر برای تبدیل سریع استفاده کن. بستن پنجره معمولاً برنامه را در tray نگه می‌دارد؛ **Exit** در منوی tray آن را کامل می‌بندد. توقف میانبرها و اجرای خودکار با شروع ویندوز هم در تنظیمات قرار دارد.

## ✨ جزئیات کاربردی

| قابلیت | نتیجه |
|:---|:---|
| 🧠 اصلاح خودکار | شواهد محلی زبان و انتخاب پیشنهاد در نتیجه‌های نامطمئن |
| ⌨️ چیدمان‌های ویندوز | خواندن فهرست چیدمان‌های ویندوز و انتخاب جفت چیدمان |
| 📋 مدیریت کلیپ‌بورد | کپی و جایگزینی متن هنگام اجرای میانبر و بازیابی محتوای پشتیبانی‌شده در شرایط امن |
| 🎨 ویرایشگر | پوسته‌ها، رابط دو زبانه، حالت جمع‌وجور، undo/redo ورودی و کاهش حرکت |
| 💾 تنظیمات | ذخیرهٔ محلی چیدمان، میانبر، پوسته و شروع برنامه |

<a id="resources"></a>

## 🪶 سبک در پس‌زمینه

در اندازه‌گیری محلی نسخهٔ **1.0.0** روی ویندوز x64، مصرف مجموع working set درخت پردازه‌ها در حالت **tray، بدون ویرایشگر باز** برابر **12.85 MiB** بود؛ ۱۰ نمونه در 11.37 ثانیه ثبت شد. پردازهٔ بومی در همین نمونه مصرف CPU بیکار صفر ثبت کرد.

وقتی ویرایشگر بسته می‌شود، WebView آن آزاد می‌شود و هستهٔ Rust و میانبرها فعال می‌مانند. باز کردن ویرایشگر، پردازه‌های WebView2 و رم بیشتری می‌خواهد؛ مقدار ۱۳ MiB مربوط به حالت پس‌زمینهٔ اندازه‌گیری‌شده است، نه سقف تضمینی همهٔ حالت‌ها.

📊 [Measurement method and results](docs/RESOURCE_USAGE.md) · [Raw samples](docs/RESOURCE_USAGE.json)

## 🔒 متن روی دستگاه خودت می‌ماند

تبدیل متن و تشخیص زبان محلی‌اند. برنامه حساب، تحلیل‌گر بازدید یا سرویس ترجمهٔ ابری ندارد و تایپ را به‌صورت پیوسته ثبت نمی‌کند. ورودی‌ها و snapshot کلیپ‌بورد موقتاً در حافظه می‌مانند. مسیر فایل تنظیمات:

<div dir="ltr">

```text
%APPDATA%\com.pimxswap.desktop\settings.json
```

</div>

[Privacy details](docs/PRIVACY.md) · [Quick start](docs/QUICKSTART.md)

## 🧩 سازگاری و نکات استفاده

چیدمان اشتباه را اصلاح می‌کند، ترجمه انجام نمی‌دهد. فیلد رمز، برنامه‌هایی با دسترسی بالاتر و برخی مدیریت‌کننده‌های خاص کلیپ‌بورد ممکن است جایگزینی خودکار را نپذیرند؛ در این حالت از ویرایشگر داخلی و paste دستی استفاده کن. بازسازی IME و بعضی کیبوردهای سفارشی محدود است. اگر میانبر عمل نکرد، کلیدهای modifier را رها و فوکوس برنامهٔ مقصد را بررسی کن. تشخیص خودکار برای انگلیسی، فارسی، عربی، روسی، آلمانی، فرانسوی و اسپانیایی پروفایل محلی دارد؛ Direct convert برای انتخاب صریح چیدمان است.

<a id="development"></a>

<details>
<summary>🛠️ توسعه‌دهندگان: سورس، ساخت و فایل نصب</summary>

برای کار روی سورس: Node.js، Rust MSVC با حداقل نسخهٔ 1.88، Visual Studio C++ Build Tools، Windows SDK و WebView2 لازم‌اند. مسیر نصب عادی کاربران در بالای صفحه است.

<div dir="ltr">

```bash
git clone https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP.git
cd PIMX_SWAP
npm ci
npm run tauri dev
# Build an NSIS installer:
npm run release
```

</div>

| مسیر | نقش |
|:---|:---|
| [`src/`](src/) | رابط React / TypeScript |
| [`src-tauri/`](src-tauri/) | هستهٔ Rust، اتصال به ویندوز و بسته‌بندی Tauri |
| [`language_profiles/`](language_profiles/) | شواهد محلی زبان |
| [`scripts/`](scripts/) | ساخت، بررسی بومی، بسته‌بندی و اندازه‌گیری |
| [`docs/`](docs/) | راهنمای استفاده، حریم خصوصی و مجوزهای وابستگی‌ها |

[Detailed source guide](docs/PROJECT_GUIDE.md) · [Tauri Windows packaging](https://v2.tauri.app/distribute/windows-installer/)

</details>

## 🤝 بازخورد

در issue، نسخهٔ برنامه، نسخهٔ ویندوز، چیدمان‌ها و مراحل بازتولید را بنویس؛ برای مثال متن ساختگی استفاده کن.

[🐛 Issues](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/issues) · [⬇️ Releases](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP/releases) · [PIMX](https://github.com/MOHAMMADREZAABEDINPOOR)

</div>
