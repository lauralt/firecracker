# Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests microvm start with command line parameter."""

import os

import pytest

import host_tools.logging as log_tools


@pytest.mark.parametrize(
    "vm_config_json",
    ["framework/vm_config.json"]
)
def test_config_json_start(test_microvm_with_ssh, vm_config_json):
    """Start a microvm using configuration sent as command line parameter.

    Create resources needed for the configuration of the microvm, then
    start a process which receives as command line parameter, one json
    for that configuration.
    """
    test_microvm = test_microvm_with_ssh

    test_microvm.create_jailed_resource(test_microvm.kernel_file,
                                        create_jail=True)
    test_microvm.create_jailed_resource(test_microvm.rootfs_file,
                                        create_jail=True)

    log_fifo_path = os.path.join(test_microvm.path, 'log_fifo')
    metrics_fifo_path = os.path.join(test_microvm.path, 'metrics_fifo')
    log_fifo = log_tools.Fifo(log_fifo_path)
    metrics_fifo = log_tools.Fifo(metrics_fifo_path)
    test_microvm.create_jailed_resource(log_fifo.path, create_jail=True)
    test_microvm.create_jailed_resource(metrics_fifo.path, create_jail=True)

    test_microvm.config_json = open(vm_config_json).read()
    test_microvm.spawn()

    response = test_microvm.machine_cfg.get()
    assert test_microvm.api_session.is_status_ok(response.status_code)
