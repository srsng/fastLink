
# import sys
from typing import List, Tuple
import pythoncom
import win32com.client as wcomcli
from win32com.shell import shell, shellcon # type: ignore


SWC_DESKTOP = 0x08
SWFO_NEEDDISPATCH = 0x01

CLSID_ShellWindows = "{9BA05972-F6A8-11CF-A442-00A0C90A8F39}"
IID_IFolderView = "{CDE725B0-CCC9-4519-917E-325D72FAB4CE}"

# https://stackoverflow.com/questions/78591447/getting-desktop-icons-positions-with-pywin32-python
def get_icon_layout() -> List[Tuple[int, int, str]]:
    shell_windows = wcomcli.Dispatch(CLSID_ShellWindows)
    hwnd = 0
    dispatch = shell_windows.FindWindowSW(
        wcomcli.VARIANT(pythoncom.VT_I4, shellcon.CSIDL_DESKTOP),
        wcomcli.VARIANT(pythoncom.VT_EMPTY, None),
        SWC_DESKTOP, hwnd, SWFO_NEEDDISPATCH,
    )
    service_provider = dispatch._oleobj_.QueryInterface(pythoncom.IID_IServiceProvider)
    browser = service_provider.QueryService(shell.SID_STopLevelBrowser, shell.IID_IShellBrowser)
    shell_view = browser.QueryActiveShellView()
    folder_view = shell_view.QueryInterface(IID_IFolderView)
    items_len = folder_view.ItemCount(shellcon.SVGIO_ALLVIEW)
    desktop_folder = shell.SHGetDesktopFolder()
    icons = []
    for i in range(items_len):
        item = folder_view.Item(i) 
        name = desktop_folder.GetDisplayNameOf([item], shellcon.SHGDN_NORMAL) # type: ignore
        pos = folder_view.GetItemPosition(item)
        icons.append((*pos, name))

    return icons
