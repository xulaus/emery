require "fnv"
require 'minitest/autorun'

describe EMERY::FNV1a do
  describe "whens given non string data" do
    it "raises an error" do
      assert_raises(RuntimeError){ EMERY::FNV1a.hexdigest(nil) }
    end
  end

  describe "whens given symbol data" do
    it "raises an error" do
      assert_equal "2da9bc21757082d4", EMERY::FNV1a.hexdigest(:adsadsa)
    end
  end

  describe "whens given string data" do
    it "returns expected hash" do
      assert_equal "2da9bc21757082d4", EMERY::FNV1a.hexdigest("adsadsa")
    end
  end
end
