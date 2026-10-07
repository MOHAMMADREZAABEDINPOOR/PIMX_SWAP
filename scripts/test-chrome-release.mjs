import { chromium } from 'playwright';
import { execFileSync } from 'node:child_process';
import { writeFile } from 'node:fs/promises';
let windowHandle=0;
const run=(action='shortcut',text='')=>execFileSync('powershell',['-NoProfile','-ExecutionPolicy','Bypass','-File','scripts/release-input.ps1',...(windowHandle?['-WindowHandle',String(windowHandle)]:['-WindowTitle','PIMXSWAP release check']),'-Action',action,'-Text',text],{windowsHide:true,encoding:'utf8'});
const browser=await chromium.launch({executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',headless:false,args:['--no-first-run','--no-default-browser-check']});
const page=await browser.newPage();
await page.context().grantPermissions(['clipboard-read','clipboard-write']);
const results=[];
try {
 await page.setContent('<title>PIMXSWAP release check</title><textarea id="editor" style="width:80vw;height:200px"></textarea><div id="rich" contenteditable="true" style="border:1px solid;height:200px"></div>');
 for(const [selector,text,expected,selected,name] of [
 ['#editor','اثممخ','hello',true,'selected Persian'],['#editor','sghl lk o,','سلام من خو',false,'caret incomplete sentence'],['#editor','sghl\nlk o,','سلام\nمن خو',true,'selected multiline'],['#rich','sghl lk o,','سلام من خو',false,'contenteditable caret']]) {
  await page.locator(selector).fill(text);await page.locator(selector).focus();
  if(selected) await page.locator(selector).press('Control+a');else await page.locator(selector).press('Control+End');
  run();console.log('focus',await page.evaluate(()=>({focused:document.hasFocus(),active:document.activeElement?.id,selection:document.activeElement?.selectionStart})));let actual;
  for(let i=0;i<70;i++){await page.waitForTimeout(100);actual=await page.locator(selector).evaluate(e=>'value' in e?e.value:e.innerText);if(actual===expected)break;}
  results.push({app:'Chrome',case:name,passed:actual===expected,actual,expected}); console.log(name,actual===expected);
 }
 windowHandle=Number(run('address','اثممخ').toString().trim());
 await page.waitForTimeout(500);run();
 let address='';
 for(let i=0;i<10;i++){await page.waitForTimeout(300);address=run('addressValue').toString().trim();if(address==='hello')break;}
 results.push({app:'Chrome',case:'address/search bar caret',passed:address==='hello',actual:address,expected:'hello'});
 console.log('address/search bar caret',address==='hello');
 await page.screenshot({path:'test-results/chrome-real.png'});
} finally {await writeFile('test-results/chrome-release.json',JSON.stringify(results,null,2));await browser.close();}
if(results.some(r=>!r.passed))process.exitCode=1;
