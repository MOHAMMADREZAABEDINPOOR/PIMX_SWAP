<div align="center">

<img src="assets/readme/hero.gif" width="1200" alt="PIMX SWAP: a sculpted bilingual keyboard with switching layout arrows" />

**[English](README.md) · [فارسی](README.fa.md)**

</div>

<div dir="rtl">

# ⌨️ PIMX SWAP

ابزار محلی اصلاح چیدمان صفحه‌کلید ویندوز با Rust، Tauri 2، React و TypeScript؛ کلید فیزیکی متن را بازسازی و به چیدمان مقصد تبدیل می‌کند: `sghl` → `سلام`.

[GitHub](https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP) · [PIMX / Profile](https://github.com/MOHAMMADREZAABEDINPOOR) · [بنر ثابت](assets/readme/hero.png)

| نمای کلی | جزئیات |
|:---|:---|
| ⌨️ تجربه | برنامه دسکتاپ ویندوز |
| 🧰 فناوری | `React` · `Vite` · `TypeScript` · `Rust` |
| 🌐 زبان راهنما | [English](README.md) · [فارسی](README.fa.md) |

[✨ امکانات](#امکانات) · [🚀 شروع کار](#شروع-کار) · [⚙️ تنظیمات](#تنظیمات) · [🌍 استقرار](#استقرار)

📖 [راهنمای تفصیلی پروژه](docs/PROJECT_GUIDE.md)

---

<a id="امکانات"></a>

## ✨ امکانات

| بخش | قابلیت موجود |
|:---|:---|
| 🌐 تجربه کاربری | اصلاح خودکار با شواهد زبان محلی و انتخاب نتیجه |
| ⌨️ کنترل | میانبر سراسری، ویرایشگر بومی و آیکون کنار ساعت |
| ⚡ روند کار | تراکنش کلیپ‌بورد با بررسی بازگردانی |
| 🌐 تجربه کاربری | رابط فارسی و انگلیسی، پوسته و تنظیم محلی |

<a id="پشته-فنی"></a>

## 🧰 پشته فنی

| ابزار | نسخه یا منبع |
|---|---|
| React | `^19.1.0` |
| Vite | `^7.1.0` |
| TypeScript | `^5.9.0` |
| Rust | `src-tauri/Cargo.toml` |
| Tauri | `2.x` |

<a id="شروع-کار"></a>

## 🚀 شروع کار

ویندوز ۱۰ یا ۱۱ x64، WebView2، Node.js 24، Rust پایدار MSVC و Visual Studio Build Tools با C++ و Windows SDK.

<div dir="ltr">

```bash
git clone https://github.com/MOHAMMADREZAABEDINPOOR/PIMX_SWAP.git
cd PIMX_SWAP

npm ci
npm run icons
npm run tauri dev
```

</div>

<a id="تنظیمات"></a>

## ⚙️ تنظیمات

فایل محیط استاندارد تعریف نشده است. برای تمرین‌های مستقل تنظیم خارجی لازم نیست؛ اگر در کد ثابت‌های سرویس یا مسیر وجود دارد، آن‌ها را پیش از اجرا بررسی کنید.

<a id="استفاده"></a>

## 🎯 استفاده

در ویندوز برای قابلیت بومی npm run tauri dev را اجرا کنید. متن را انتخاب و Ctrl+Shift+Space بزنید؛ Ctrl+Alt+Shift+Space جفت چیدمان را تبدیل و Ctrl+Alt+P ویرایشگر را باز می‌کند. میانبر قابل تغییر است.

<a id="ساختار-پروژه"></a>

## 🗂️ ساختار پروژه

| مسیر | نقش |
|---|---|
| [`assets/`](assets/) | فایل برند، رسانه و README |
| [`docs/`](docs/) | راهنمای تکمیلی |
| [`language_profiles/`](language_profiles/) | شواهد زبان محلی |
| [`public/`](public/) | فایل عمومی وب |
| [`scripts/`](scripts/) | ابزار توسعه و نگهداری |
| [`src/`](src/) | کد برنامه |
| [`src-tauri/`](src-tauri/) | هسته بومی Rust |
| [`index.html`](index.html) | فایل ورودی یا تنظیم پروژه |
| [`package.json`](package.json) | فایل ورودی یا تنظیم پروژه |
| [`tsconfig.json`](tsconfig.json) | فایل ورودی یا تنظیم پروژه |

<a id="فرمان‌ها-و-بررسی"></a>

## 🧪 فرمان‌ها و بررسی

| فرمان | کاربرد |
|:---|:---|
| `npm run dev` | 🧑‍💻 سرور توسعه |
| `npm run build` | 📦 ساخت نسخه انتشار |
| `npm run preview` | 👀 پیش‌نمایش خروجی |
| `npm run tauri` | 🖥️ ابزار Tauri |

<div dir="ltr">

```bash
npm run dev
npm run build
npm run preview
npm run test:ui
npm run icons
npm run test:workspace
```

</div>

این‌ها فرمان‌های موجود در package.json هستند؛ فهرست بالا گزارش اجرای آزمون نیست. فرمان تست ممکن است مرورگر، سرویس یا دیتابیس آماده بخواهد.

<a id="استقرار"></a>

## 🌍 استقرار

scripts/release.ps1 فرانت‌اند و کد بومی را بررسی و نصب‌کننده NSIS می‌سازد. اسناد docs/ مسیر انتشار را توضیح می‌دهند؛ خروجی build و release در Git قرار نمی‌گیرد.

<a id="محدودیت‌ها"></a>

## 📌 محدودیت‌ها

ویندوز ۱۰ یا ۱۱، WebView2 و ابزار ساخت بومی لازم است. IME و ترکیب دلخواه dead key کامل بازسازی نمی‌شود. برنامه محافظت‌شده ممکن است جایگزینی را نپذیرد؛ از ویرایشگر استفاده کنید. پیش‌نمایش مرورگر قابلیت بومی ندارد.

<a id="رفع-مشکل"></a>

## 🛠️ رفع مشکل

- میانبر رزروشده: ترکیب دیگری در تنظیمات انتخاب کنید.
- چیدمان غایب: در تنظیمات ویندوز اضافه و فهرست را تازه کنید.
- قابلیت بومی در مرورگر نیست: tauri dev یا برنامه ساخته‌شده را اجرا کنید.

<a id="مشارکت"></a>

## 🤝 مشارکت

برای تغییر، شاخه مستقل بسازید، رفتار فعلی را بررسی کنید و توضیح روشن همراه تغییر بفرستید. اطلاعات خصوصی، خروجی build و دیتابیس محلی را commit نکنید.

راهنماهای همراه:

- [docs/architecture.md](docs/architecture.md)
- [docs/development.md](docs/development.md)
- [docs/PRIVACY.md](docs/PRIVACY.md)
- [docs/validation.md](docs/validation.md)
- [docs/QUICKSTART.md](docs/QUICKSTART.md)
- [docs/release-checklist.md](docs/release-checklist.md)

<a id="مجوز"></a>

## 📄 مجوز

فایل مجوز در این نسخه موجود نیست. نمایش عمومی کد به‌تنهایی مجوز استفاده مجدد نیست؛ برای شرایط استفاده با مالک مخزن هماهنگ کنید.

---

ساخته‌شده در مجموعه **PIMX** · مستندات فارسی و انگلیسی.

---

<div align="center">

⌨️ **PIMX SWAP** · [English](README.md) · [فارسی](README.fa.md)

</div>

</div>
