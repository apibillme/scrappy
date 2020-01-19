#!/usr/bin/env python
import os
import json
import subprocess

def sort_by_dep(unsorted):
    sorted = []
    unsorted = dict(unsorted)
    while unsorted:
        for node, edges in unsorted.items():
            for edge in edges:
                if edge in unsorted:
                    break
            else:
                del unsorted[node]
                sorted.append((node, edges))
    return sorted


def load_metadata():
    return json.loads(subprocess.Popen(
        'cargo metadata --no-deps --format-version=1',
        shell=True, stdout=subprocess.PIPE).communicate()[0])


def get_packages():
    metadata = load_metadata()
    graph = []
    for pkg in metadata['packages']:
        graph.append((
            pkg['name'],
            [x['name'] for x in pkg['dependencies'] if x['name'].startswith('symbolic-')]
        ))
    return sort_by_dep(graph)

for pkg, deps in get_packages():
    # this is a bit of a hack
    if os.path.isdir(pkg):
        subprocess.Popen(['cargo', 'publish'], cwd=pkg).wait()
    else:
        subprocess.Popen(['cargo', 'publish']).wait()
        