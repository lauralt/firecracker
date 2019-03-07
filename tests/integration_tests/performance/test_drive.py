# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests for drive performance with fio."""

import os
import re

import host_tools.drive as drive_tools
import host_tools.network as net_tools   # pylint: disable=import-error

MAX_IOPS = 150000


def test_drive_performance(test_microvm_with_ssh, network_config):
    """Test the output of fio for io operations on /dev/vdb"""
    test_microvm = test_microvm_with_ssh
    test_microvm.spawn()

    test_microvm.basic_config(vcpu_count=1, ht_enabled=False, mem_size_mib=1024)
    _tap, _, _ = test_microvm.ssh_network_config(network_config, '1')

    fs1 = drive_tools.FilesystemFile(
        os.path.join(test_microvm.fsfiles, 'scratch'), size=2048
    )
    response = test_microvm.drive.put(
        drive_id='scratch',
        path_on_host=test_microvm.create_jailed_resource(fs1.path),
        is_root_device=False,
        is_read_only=False
    )

    assert test_microvm.api_session.is_status_no_content(response.status_code)

    test_microvm.start()
    ssh_connection = net_tools.SSHConnection(test_microvm.ssh_config)

    _, stdout, stderr = ssh_connection.execute_command("lscpu")
    assert stderr.read().decode("utf-8") == ''
     #fio_config = 'integration_tests/performance/config.fio'
     #fio_config_remote = 'config.fio'
     #ssh_connection.scp_file(fio_config, fio_config_remote)
    #_, stdout, stderr = ssh_connection.execute_command("fio config.fio")

#    assert stderr.read().decode('utf-8') == ''

    #for _ in range(5):
    #    stdout.readline()

    #line = stdout.readline()
    #dict = {'io':'524288KB', 'bw':'595782KB/s', 'iops':150000}
    #fio_params = line.replace(" ","").split(":")
    #fio_params_list = fio_params[1].split(",")
    #[key, value] = list(map(lambda x: x.strip(), fio_params_list.split('=')))

    # ssh_connection.close()
    #dict_max_values = {'bw': 600000, 'iops': 150000, 'runt': 1600}

    # f = open('integration_tests/performance/results_fio', 'r')
    # for _ in range(5):
    #     f.readline()
    # line = f.readline()
    # fio_params = line.replace(" ","").split(":")
    # fio_params_list = fio_params[1].split(",")
    # for fio_par in fio_params_list:
    #     [key, value] = list(map(lambda x: x.strip(), fio_par.split('=')))
    #     if key in dict_max_values.keys():
    #         value = re.sub("[^0-9]", "", value)
    #         assert int(value) < dict_max_values[key]

    #f.close()
