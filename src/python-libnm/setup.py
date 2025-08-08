# SPDX-License-Identifier: LGPL-2.1-or-later

import setuptools


def requirements():
    req = []
    with open("requirements.txt") as fd:
        for line in fd:
            line.strip()
            if not line.startswith("#"):
                req.append(line)
    return req


setuptools.setup(
    name="nmstate",
    version="0.1.0",
    author="Gris Ge",
    author_email="fge@redhat.com",
    description="Python binding of NetworkManager",
    long_description="Python binding of NetworkManager",
    url="https://github.com/cathay4t/NetworkManager/",
    packages=setuptools.find_packages(),
    install_requires=requirements(),
    license="ASL2.0+",
    python_requires=">=3.10",
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: POSIX :: Linux",
    ],
)
