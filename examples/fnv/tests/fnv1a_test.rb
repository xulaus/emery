require "fnv"
require 'minitest/autorun'

describe Digest::FNV1a64 do
  describe "whens given non string data" do
    it "raises an error" do
      assert_raises(RuntimeError){ Digest::FNV1a64.hexdigest(nil) }
    end
  end

  describe "whens given symbol data" do
    it "raises an error" do
      assert_equal "2da9bc21757082d4", Digest::FNV1a64.hexdigest(:adsadsa)
    end
  end

  describe "whens given string data" do
    it "returns expected hash" do
      assert_equal "2da9bc21757082d4", Digest::FNV1a64.hexdigest("adsadsa")
    end
  end
end

describe Digest::FNV1a128 do
  describe "whens given non string data" do
    it "raises an error" do
      assert_raises(RuntimeError){ Digest::FNV1a128.hexdigest(nil) }
    end
  end

  describe "whens given symbol data" do
    it "raises an error" do
      assert_equal "7dd10dc6da4ff78da7c74cad91c6ee6c", Digest::FNV1a128.hexdigest(:adsadsa)
    end
  end

  describe "whens given string data" do
    it "returns expected hash" do
      assert_equal "7dd10dc6da4ff78da7c74cad91c6ee6c", Digest::FNV1a128.hexdigest("adsadsa")
    end
  end
end
