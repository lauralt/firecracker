# Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests microvm start with configuration file as command line parameter."""

import os
import re
import time

from retry.api import retry_call
from subprocess import run

import pytest

import framework.utils as utils

import host_tools.drive as drive_tools
import host_tools.logging as log_tools
import host_tools.network as net_tools


def _configure_vm_from_json(test_microvm, vm_config_file):
    """Configure a microvm using a file sent as command line parameter.

    Create resources needed for the configuration of the microvm and
    set as configuration file a copy of the file that was passed as
    parameter to this helper function.
    """
    test_microvm.create_jailed_resource(test_microvm.kernel_file,
                                        create_jail=True)
    test_microvm.create_jailed_resource(test_microvm.rootfs_file,
                                        create_jail=True)

    # vm_config_file is the source file that keeps the desired vmm
    # configuration. vm_config_path is the configuration file we
    # create inside the jail, such that it can be accessed by
    # firecracker after it starts.
    vm_config_path = os.path.join(test_microvm.path,
                                  os.path.basename(vm_config_file))
    with open(vm_config_file) as f1:
        with open(vm_config_path, "w") as f2:
            for line in f1:
                f2.write(line)
    test_microvm.create_jailed_resource(vm_config_path, create_jail=True)
    test_microvm.jailer.extra_args = {'config-file': os.path.basename(
        vm_config_file)}


@pytest.mark.parametrize(
    "vm_config_file",
    ["framework/vm_config_perf.json"]
)
def test_config_start(test_microvm_with_api, vm_config_file):
    """Test if a microvm configured from file boots successfully."""
    test_microvm = test_microvm_with_api
    if test_microvm_with_api.jailer.netns:
        run('ip netns add {}'.format(test_microvm_with_api.jailer.netns),
            shell=True, check=True)

    # 3 drives (+ root device)
    fs1 = drive_tools.FilesystemFile(
        os.path.join(test_microvm.fsfiles, 'id1')
    )
    test_microvm.create_jailed_resource(fs1.path, create_jail=True)
    fs2 = drive_tools.FilesystemFile(
        os.path.join(test_microvm.fsfiles, 'id2')
    )
    test_microvm.create_jailed_resource(fs2.path, create_jail=True)
    fs3 = drive_tools.FilesystemFile(
        os.path.join(test_microvm.fsfiles, 'id3')
    )
    test_microvm.create_jailed_resource(fs3.path, create_jail=True)

    # 3 net devices
    first_if_name = 'first_tap'
    _tap1 = net_tools.Tap(first_if_name, test_microvm.jailer.netns)
    second_if_name = 'second_tap'
    _tap2 = net_tools.Tap(second_if_name, test_microvm.jailer.netns)
    third_if_name = 'third_tap'
    _tap3 = net_tools.Tap(third_if_name, test_microvm.jailer.netns)

    _configure_vm_from_json(test_microvm, vm_config_file)
    start = time.time()
    test_microvm.spawn(add_netns=False)
    end = time.time()
    # elapsed time measured in ms
    elapsed_time = (end - start) * 1000

    f = open("framework/results_config_file_local", "a")
    f.write("%.3f\n" % elapsed_time)

    response = test_microvm.machine_cfg.get()
    assert test_microvm.api_session.is_status_ok(response.status_code)


@pytest.mark.parametrize(
    "vm_config_file",
    ["framework/vm_config.json"]
)
def test_config_start_with_api(test_microvm_with_ssh, vm_config_file):
    """Test if a microvm configured from file boots successfully."""
    test_microvm = test_microvm_with_ssh

    _configure_vm_from_json(test_microvm, vm_config_file)
    test_microvm.spawn()

    response = test_microvm.machine_cfg.get()
    assert test_microvm.api_session.is_status_ok(response.status_code)


@pytest.mark.parametrize(
    "vm_config_file",
    ["framework/vm_log_config.json"]
)
def test_config_start_no_api(test_microvm_with_ssh, vm_config_file):
    """Test microvm start when API server thread is disabled."""
    test_microvm = test_microvm_with_ssh

    log_fifo_path = os.path.join(test_microvm.path, 'log_fifo')
    log_fifo = log_tools.Fifo(log_fifo_path)
    test_microvm.create_jailed_resource(log_fifo.path, create_jail=True)

    _configure_vm_from_json(test_microvm, vm_config_file)
    test_microvm.jailer.extra_args.update({'no-api': None})

    test_microvm.spawn()

    # Get Firecracker PID so we can check the names of threads.
    firecracker_pid = test_microvm.jailer_clone_pid

    # Get names of threads in Firecracker.
    cmd = 'ps -T --no-headers -p {} | awk \'{{print $5}}\''.format(
        firecracker_pid
    )

    # Retry running 'ps' in case it failed to list the firecracker process
    # The regex matches any expression that contains 'firecracker' and does
    # not contain 'fc_api'
    retry_call(
        utils.search_output_from_cmd,
        fkwargs={
            "cmd": cmd,
            "find_regex": re.compile("^(?!.*fc_api)(?:.*)?firecracker",
                                     re.DOTALL)
            },
        exceptions=RuntimeError,
        tries=10,
        delay=1)

    # Check that microvm was successfully booted.
    lines = log_fifo.sequential_reader(1)
    assert lines[0].startswith('Running Firecracker')
