import dataclasses
import os
import re
from dataclasses import dataclass
from typing import Set

import fire


@dataclass
class EntryState:
    mime: Set = dataclasses.field(default_factory=set)
    ext: Set = dataclasses.field(default_factory=set)
    apple: Set = dataclasses.field(default_factory=set)
    strength: Set = dataclasses.field(default_factory=set)

    def assert_unique(self, filepath, line_no):
        for f in [self.strength]:
            if len(f) >= 2:
                print(f'Found duplicated aux info '
                      f'at {filepath}:{line_no}: {f}!')


def noop(_line, _line_no, _filepath):
    pass


def check_line(folder=None, fn_aux=noop, fn_magic=noop, fn_magic_entry=noop):
    folder = folder or r"/usr/share/file/magic"
    for file in os.listdir(folder):
        filepath = os.path.join(folder, file)
        with open(filepath, 'rb') as f:
            for line_no, line in enumerate(f.readlines()):
                line = line.strip()
                if line == b'':
                    continue
                elif line.startswith(b'#'):
                    continue
                elif line.startswith(b'!:'):
                    fn_aux(line, line_no, filepath)
                elif line.startswith(b'>'):
                    fn_magic(line, line_no, filepath)
                else:
                    fn_magic_entry(line, line_no, filepath)
                    fn_magic(line, line_no, filepath)


def check_aux_line(folder=None):
    class T:
        entry = EntryState()

        def fn_aux(self, line, line_no, filepath):
            d = self.entry.__dict__
            for k in d.keys():
                if line.startswith(b'!:' + k.encode('utf-8')):
                    getattr(self.entry, k).add(line)
            self.entry.assert_unique(filepath, line_no)

        def fn_magic_entry(self, _line, _line_no, _filepath):
            self.entry = EntryState()

    t = T()
    check_line(folder, fn_aux=t.fn_aux, fn_magic_entry=t.fn_magic_entry)


def check_type_mask(folder=None):
    class T:
        def __init__(self):
            self.typs = set()

        def fn_magic(self, line: bytes, line_no, filepath):
            line = line.decode('utf-8', errors='ignore')
            # print(f'{filepath}:{line_no} - {line}')
            part = line.split()[1]
            # typs = list(re.split(r'[+\-*/&^%|]', part))
            #
            # self.typs.add(typs[0])
            if 'use' in part and 'database' in filepath:
                # self.typs.add(part.split('/')[0])
                print(line)

    t = T()
    check_line(folder, fn_magic=t.fn_magic)
    print(t.typs)


def check_reln_val_sign(folder=None):
    def fn_magic(line: bytes, line_no, filepath):
        line = line.decode('utf-8', errors='ignore')
        part = line.split()[2]
        if len(part) >= 2 and part[1] == '-':
            print(line)

    check_line(folder, fn_magic=fn_magic)


def check_if_contains_illegal_chars():
    folder = r'/usr/share/file/magic'
    for file in os.listdir(folder):
        file = os.path.join(folder, file)
        with open(file, 'rb') as f:
            for n, l in enumerate(f.readlines()):
                try:
                    l.decode(encoding='utf-8')
                except UnicodeDecodeError as e:
                    print(e)
                    print(f'{file}:{n}: l.strip()')


if __name__ == '__main__':
    fire.Fire(check_type_mask)
