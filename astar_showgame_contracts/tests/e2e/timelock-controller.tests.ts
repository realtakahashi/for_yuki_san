import { bnArg, expect, getSigners } from './helpers'
import Constructors from '../../typechain-generated/constructors/my_timelock_controller'
import Contract from '../../typechain-generated/contracts/my_timelock_controller'
import { ApiPromise } from '@polkadot/api'

function getMessageAbi(contract: Contract, identifier: string) {
  return contract.abi.findMessage(identifier)!
}

describe('MY_TIMELOCK_CONTROLLER', () => {
  async function setup() {
    const api = await ApiPromise.create()

    const signers = getSigners()
    const bob = signers[1]
    const defaultSigner = signers[0]

    const contractFactory = new Constructors(api, defaultSigner)
    const { address: contractAddress } = await contractFactory.new(0, [bob.address], [bob.address])

    const contract = new Contract(contractAddress, defaultSigner, api)

    // const contract = await setupContract('my_timelock_controller', 'new', 0, [bob.address], [bob.address])

    return { api, contract, bob, alice: defaultSigner }
  }

  it('TIMELOCK CONTROLLER - can schedule', async () => {
    const { api, contract, bob } = await setup()

    // Arrange - Prepare data for schedule
    const transaction = {
      callee: contract.address,
      selector: [0, 0, 0, 0],
      input: [],
      transferredValue: 0,
      gasLimit: 0
    }
    const salt = bnArg(0)

    // Act - Bob scheduled the transaction
    const id = (await contract.query.hashOperation(transaction, null, salt)).value.unwrapRecursively()
    expect((await contract.query.isOperationPending(id)).value.unwrapRecursively()).to.be.eq(false)
    await contract.withSigner(bob).tx.schedule(transaction, null, salt, 0)

    // Assert - Operation must be scheduled, it should be in Pending state and in Ready state(because min delay is zero)
    expect((await contract.query.isOperationPending(id)).value.unwrapRecursively()).to.be.eq(true)
    expect((await contract.query.isOperationReady(id)).value.unwrapRecursively()).to.be.eq(true)
    expect((await contract.query.isOperationDone(id)).value.unwrapRecursively()).to.be.eq(false)

    await api.disconnect()
  })

  it('TIMELOCK CONTROLLER - schedule and execute without input data `TimelockController::get_min_delay`', async () => {
    const { api, contract, bob } = await setup()

    // Arrange - Prepare data for execute `get_min_delay`
    const message = getMessageAbi(contract, 'TimelockController::get_min_delay')
    const transaction = {
      callee: contract.address,
      selector: message.selector.toU8a() as unknown as number[],
      input: [],
      transferredValue: 0,
      gasLimit: 0
    }
    const salt = bnArg(0)

    // Act - Bob scheduled the transaction
    const id = (await contract.query.hashOperation(transaction, null, salt)).value.unwrapRecursively()
    await contract.withSigner(bob).tx.schedule(transaction, null, salt, 0)

    // Assert - Transaction must be updated and now the state is Done
    await expect(contract.query.isOperationDone(id)).to.have.output(false)
    await contract.withSigner(bob).tx.execute(transaction, null, salt)
    await expect(contract.query.isOperationDone(id)).to.have.output(true)

    await api.disconnect()
  })

  it('TIMELOCK CONTROLLER - schedule and execute by passing value into `TimelockController::update_delay`, and update', async () => {
    const { api, contract, bob } = await setup()

    // Arrange - Prepare data for execute `update_delay` with a new `min_delay`
    const message = getMessageAbi(contract, 'TimelockController::update_delay')
    const new_min_delay = 15

    const transaction = {
      callee: contract.address,
      selector: Array.from(message.selector.toU8a()),
      input: Array.from(api.createType('u64', new_min_delay).toU8a()),
      transferredValue: 0,
      gasLimit: 0
    }
    const salt = bnArg(0)

    // Act - Bob scheduled the transaction
    await contract.withSigner(bob).tx.schedule(transaction, null, salt, 0)

    // Assert - Min delay must be updated via `execute` method
    await expect(contract.query.getMinDelay()).to.have.output(0)
    await contract.withSigner(bob).tx.execute(transaction, null, salt)
    await expect(contract.query.getMinDelay()).to.have.output(new_min_delay)

    await api.disconnect()
  })

  it('TIMELOCK CONTROLLER - fails schedule because signer is not proposal', async () => {
    const { api, contract, alice } = await setup()

    // Arrange - Prepare data for schedule
    const transaction = {
      callee: contract.address,
      selector: [0, 0, 0, 0],
      input: [],
      transferredValue: 0,
      gasLimit: 0
    }
    const salt = bnArg(0)

    // Assert - Alice can't schedule the transaction
    await expect(contract.withSigner(alice).tx.schedule(transaction, null, salt, 0)).to.eventually.be.rejected

    await api.disconnect()
  })

  it('TIMELOCK CONTROLLER - fails execute because signer is not executor', async () => {
    const { api, contract, bob, alice } = await setup()

    // Arrange - Prepare data for schedule
    const transaction = {
      callee: contract.address,
      selector: [0, 0, 0, 0],
      input: [],
      transferredValue: 0,
      gasLimit: 0
    }
    const salt = bnArg(0)

    // Act - Bob scheduled the transaction
    await contract.withSigner(bob).tx.schedule(transaction, null, salt, 0)

    // Assert - Alice can't execute the transaction
    await expect(contract.withSigner(alice).tx.execute(transaction, null, salt)).to.eventually.be.rejected

    await api.disconnect()
  })

  it('TIMELOCK CONTROLLER - fails update_delay', async () => {
    const { api, contract, bob } = await setup()

    // Assert - Bob is not contract itself, then it must fails
    await expect(contract.withSigner(bob).tx.updateDelay(15)).to.eventually.be.rejected

    await api.disconnect()
  })
})
