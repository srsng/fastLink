# import sys
from py_get_layout.icon import get_icon_layout

if __name__ == "__main__":
    # print(
    #     "Python {:s} {:03d}bit on {:s}\n".format(
    #         " ".join(elem.strip() for elem in sys.version.split("\n")),
    #         64 if sys.maxsize > 0x100000000 else 32,
    #         sys.platform,
    #     )
    # )
    icons = get_icon_layout()
    for i, icon in enumerate(icons):
        print(f"{icon[0]} {icon[1]} {icon[2]}")