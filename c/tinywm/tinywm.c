
/* TinyWM is written by Nick Welch <nick@incise.org> in 2005 & 2011.
 *
 * This software is in the public domain
 * and is provided AS IS, with NO WARRANTY. */

#include <X11/Xlib.h>
#include <X11/keysym.h>
#include <stdlib.h>
#include <stdio.h>

#define MAX(a, b) ((a) > (b) ? (a) : (b))

int main(void)
{
    Display * dpy;
    XWindowAttributes attr;
    XButtonEvent start;
    XEvent ev;

    if(!(dpy = XOpenDisplay(0x0))) return 1;

    XGrabKey(dpy, XKeysymToKeycode(dpy, XK_Return), Mod1Mask,
            DefaultRootWindow(dpy), True, GrabModeAsync, GrabModeAsync);
    XGrabKey(dpy, XKeysymToKeycode(dpy, XK_Q), Mod1Mask,
            DefaultRootWindow(dpy), True, GrabModeAsync, GrabModeAsync);
    XGrabKey(dpy, XKeysymToKeycode(dpy, XK_p), Mod1Mask,
            DefaultRootWindow(dpy), True, GrabModeAsync, GrabModeAsync);
    XGrabButton(dpy, 1, Mod1Mask, DefaultRootWindow(dpy), True,
            ButtonPressMask|ButtonReleaseMask|PointerMotionMask, GrabModeAsync, GrabModeAsync, None, None);
    XGrabButton(dpy, 2, Mod1Mask, DefaultRootWindow(dpy), True,
            ButtonPressMask|ButtonReleaseMask|PointerMotionMask, GrabModeAsync, GrabModeAsync, None, None);
    XGrabButton(dpy, 3, Mod1Mask, DefaultRootWindow(dpy), True,
            ButtonPressMask|ButtonReleaseMask|PointerMotionMask, GrabModeAsync, GrabModeAsync, None, None);

    start.subwindow = None;
    for(;;)
    {
        XNextEvent(dpy, &ev);
        if(ev.type == KeyPress)
        {
            if (ev.xkey.subwindow != None) {
                if(ev.xkey.keycode == XKeysymToKeycode(dpy, XK_Return))
                    XRaiseWindow(dpy, ev.xkey.subwindow);
            }
            if(ev.xkey.keycode == XKeysymToKeycode(dpy, XK_Q))
                break;
            if(ev.xkey.keycode == XKeysymToKeycode(dpy, XK_p))
                system("/bin/sh -c dmenu_run");
        }
        else if(ev.type == ButtonPress && ev.xbutton.subwindow != None)
        {
            if (ev.xbutton.button == 2)
                XRaiseWindow(dpy, ev.xbutton.subwindow);
            else 
            {
                XGetWindowAttributes(dpy, ev.xbutton.subwindow, &attr);
                start = ev.xbutton;
            }
        }
        else if(ev.type == MotionNotify && start.subwindow != None)
        {
            int x_diff = ev.xbutton.x_root - start.x_root;
            int y_diff = ev.xbutton.y_root - start.y_root;
            XMoveResizeWindow(dpy, start.subwindow,
                attr.x + (start.button==1 ? x_diff : 0),
                attr.y + (start.button==1 ? y_diff : 0),
                MAX(1, attr.width + (start.button==3 ? x_diff : 0)),
                MAX(1, attr.height + (start.button==3 ? y_diff : 0)));
        }
        else if(ev.type == ButtonRelease)
            start.subwindow = None;
    }
}