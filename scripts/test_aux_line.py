import dataclasses
import os
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


def check_aux_line(folder=None):
    folder = folder or r"/usr/share/file/magic"
    for file in os.listdir(folder):
        filepath = os.path.join(folder, file)
        with open(filepath, 'rb') as f:
            entry = EntryState()
            for line_no, line in enumerate(f.readlines()):
                line = line.strip()
                if line == b'':
                    continue
                elif line.startswith(b'#'):
                    continue
                elif line.startswith(b'!:'):
                    d = entry.__dict__
                    for k in d.keys():
                        if line.startswith(b'!:' + k.encode('utf-8')):
                            getattr(entry, k).add(line)
                    entry.assert_unique(filepath, line_no)
                elif line.startswith(b'>'):
                    continue
                else:
                    # print(line.decode('utf-8', 'ignore'))
                    entry = EntryState()


if __name__ == '__main__':
    fire.Fire(check_aux_line)
