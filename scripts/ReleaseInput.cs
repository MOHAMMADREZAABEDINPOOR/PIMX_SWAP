using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class ReleaseInput {
 [StructLayout(LayoutKind.Explicit, Size=40)] public struct Input {
  [FieldOffset(0)] public uint type; [FieldOffset(8)] public ushort key; [FieldOffset(10)] public ushort scan; [FieldOffset(12)] public uint flags; [FieldOffset(16)] public uint time; [FieldOffset(24)] public IntPtr extra;
 }
 [DllImport("user32.dll")] static extern uint SendInput(uint n,Input[] input,int size);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr w,IntPtr p);
 [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")] static extern bool AttachThreadInput(uint a,uint b,bool attach);
 [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr w);
 [DllImport("user32.dll")] static extern bool ShowWindowAsync(IntPtr w,int n);
 public static void Focus(long h) {
  var w=new IntPtr(h); var a=GetCurrentThreadId();var b=GetWindowThreadProcessId(GetForegroundWindow(),IntPtr.Zero);var t=GetWindowThreadProcessId(w,IntPtr.Zero);
  bool old=a!=b&&AttachThreadInput(a,b,true);bool target=t!=a&&t!=b&&AttachThreadInput(a,t,true);
  ShowWindowAsync(w,9);System.Threading.Thread.Sleep(100);SetForegroundWindow(w);
  if(target)AttachThreadInput(a,t,false);if(old)AttachThreadInput(a,b,false);
  if(GetForegroundWindow()!=w){Keys(0x12);SetForegroundWindow(w);}
 }
 public static long Foreground() {return GetForegroundWindow().ToInt64();}
 public static void Keys(ushort key,bool control=false,bool shift=false,bool win=false) {
  var list=new List<Input>();var mods=new List<ushort>();if(control)mods.Add(0x11);if(shift)mods.Add(0x10);if(win)mods.Add(0x5b);
  foreach(var m in mods)list.Add(new Input{type=1,key=m,flags=(uint)(m==0x5b?1:0)});
  list.Add(new Input{type=1,key=key});list.Add(new Input{type=1,key=key,flags=2});
  mods.Reverse();foreach(var m in mods)list.Add(new Input{type=1,key=m,flags=(uint)(m==0x5b?3:2)});
  if(SendInput((uint)list.Count,list.ToArray(),40)!=list.Count)throw new Exception("SendInput blocked");
 }
 public static void Text(string text) {var list=new List<Input>();foreach(var c in text){list.Add(new Input{type=1,scan=c,flags=4});list.Add(new Input{type=1,scan=c,flags=6});} if(SendInput((uint)list.Count,list.ToArray(),40)!=list.Count)throw new Exception("Unicode input blocked");}
}
