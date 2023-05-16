import {expect, getSigners} from '../helpers'
import {ApiPromise} from '@polkadot/api'
import ConstructorsPSP34 from '../../../typechain-generated/constructors/my_psp34'
import ContractPSP34 from '../../../typechain-generated/contracts/my_psp34'
import * as PSP34Returns from '../../../typechain-generated/types-returns/my_psp34'
import * as PSP34Args from '../../../typechain-generated/types-arguments/my_psp34'
import {addressToU8a} from '@polkadot/util-crypto/address/util'

describe('MY_PSP34', () => {
  async function setup() {
    const api = await ApiPromise.create()

    const signers = getSigners()
    const defaultSigner = signers[2]
    const alice = signers[0]
    const bob = signers[1]

    const contractFactory = new ConstructorsPSP34(api, defaultSigner)
    const contractAddress = (await contractFactory.new()).address
    const contract = new ContractPSP34(contractAddress, defaultSigner, api)

    return {
      api,
      defaultSigner,
      alice,
      bob,
      contract,
      query: contract.query,
      tx: contract.tx,
      close: async () => {
        await api.disconnect()
      }
    }
  }

  it('Return collection_id of account', async () => {
    const { query, contract, close } = await setup()

    const expected_collection_id = PSP34Returns.IdBuilder.Bytes(addressToU8a(contract.address) as unknown as number[])
    const actual_collection_id = await query.collectionId()
    expect(expected_collection_id).to.have.output(actual_collection_id)

    await close()
  })

  it('Returns total supply', async () => {
    const {
      query,
      tx,
      close
    } = await setup()

    await expect(query.totalSupply()).to.have.bnToNumber(0)
    await tx.mintToken()
    await tx.mintToken()
    await tx.mintToken()

    await expect(query.totalSupply()).to.have.bnToNumber(3)

    await close()
  })

  it('Transfer works', async () => {
    const {
      contract,
      defaultSigner: sender,
      alice,
      query,
      tx,
      close
    } = await setup()

    await tx.mintToken()

    await expect(query.balanceOf(sender.address)).to.have.output(1)
    await expect(query.balanceOf(alice.address)).to.have.output(0)

    await contract.tx.transfer(alice.address, PSP34Args.IdBuilder.U8(0), [])

    await expect(query.balanceOf(sender.address)).to.have.output(0)
    await expect(query.balanceOf(alice.address)).to.have.output(1)

    await close()
  })

  it('Approved transfer works', async () => {
    const {
      contract,
      defaultSigner: sender,
      query,
      tx,
      alice,
      close
    } = await setup()

    await tx.mintToken()

    await expect(query.balanceOf(sender.address)).to.have.output(1)
    await expect(query.balanceOf(alice.address)).to.have.output(0)

    const token_id = PSP34Args.IdBuilder.U8(0)

    // Approve only transfer for token 1
    await contract.tx.approve(alice.address, token_id, true)

    await contract.withSigner(alice).tx.transfer(alice.address, token_id, [])

    await expect(query.balanceOf(sender.address)).to.have.output(0)
    await expect(query.balanceOf(alice.address)).to.have.output(1)

    await close()
  })

  it('Approved operator transfer works', async () => {
    const {
      contract,
      defaultSigner: sender,
      alice,
      query,
      tx,
      close
    } = await setup()

    await tx.mintToken()

    await expect(query.balanceOf(sender.address)).to.have.output(1)
    await expect(query.balanceOf(alice.address)).to.have.output(0)
    // Approved transfer for any token
    await contract.tx.approve(alice.address, null, true)

    await contract.withSigner(alice).tx.transfer(alice.address, PSP34Args.IdBuilder.U8(0), [])

    await expect(query.balanceOf(sender.address)).to.have.output(0)
    await expect(query.balanceOf(alice.address)).to.have.output(1)

    await close()
  })

  it('PSP34 - transfer works', async () => {
    const {
      tx,
      query,
      defaultSigner: sender,
      bob,
      close: closePSP34
    } = await setup()
    // Arrange - Sender mint a Token
    await tx.mintToken()
    await expect(query.ownerOf(PSP34Args.IdBuilder.U8(0))).to.have.output(sender.address)

    // Act - Alice transfers the token form sender to bob
    await tx.transfer(bob.address, PSP34Args.IdBuilder.U8(0), 'data' as unknown as string[])
    // Assert - Bob is now owner of the token
    await expect(query.ownerOf(PSP34Args.IdBuilder.U8(0))).to.have.output(bob.address.toString())

    await closePSP34()
  })

  it('Can nextot transfer non-existing token', async () => {
    const {
      contract,
      alice: receiver,
      defaultSigner: sender,
      query,
      close
    } = await setup()

    await expect(query.balanceOf(sender.address)).to.have.output(0)

    await expect(contract.tx.transfer(receiver.address, PSP34Args.IdBuilder.U8(0), [])).to.eventually.be.rejected

    await expect(query.balanceOf(sender.address)).to.have.output(0)

    await close()
  })

  it('Can not transfer without allowance', async () => {
    const {
      contract,
      alice,
      defaultSigner: sender,
      query,
      tx,
      close
    } = await setup()

    await tx.mintToken()
    await expect(query.balanceOf(sender.address)).to.have.output(1)

    await expect(contract.withSigner(alice).tx.transfer(alice.address, PSP34Args.IdBuilder.U8(0), []))
      .to.eventually.be.rejected

    await expect(query.balanceOf(sender.address)).to.have.output(1)

    await close()
  })

  it('Can mint any Id', async () => {
    const {
      contract,
      alice,
      defaultSigner: sender,
      query,
      tx,
      close
    } = await setup()

    const ids = [
      PSP34Args.IdBuilder.U8(123),
      PSP34Args.IdBuilder.U16(123),
      PSP34Args.IdBuilder.U32(123),
      PSP34Args.IdBuilder.U64(123),
      PSP34Args.IdBuilder.U128(123),
      PSP34Args.IdBuilder.Bytes(['1', '2', '3'])
    ]

    let index = 0
    for (const id of ids) {
      await expect(query.balanceOf(sender.address)).to.have.output(index)
      await expect(query.ownerOf(id)).to.have.output(null)
      await tx.mint(id)
      await expect(query.ownerOf(id)).to.have.output(sender.address)
      index++
    }

    await expect(query.balanceOf(sender.address)).to.have.output(6)

    await close()
  })
})
